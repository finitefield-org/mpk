package webapp

import (
	"bytes"
	"context"
	"errors"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestReserveOrderAcceptsRequestWithinBalance(t *testing.T) {
	ledger := &fakeLedger{}
	handler := Handler{
		Wallet: fakeWallet{balanceCents: 2500},
		Ledger: ledger,
	}

	recorder := httptest.NewRecorder()
	request := httptest.NewRequest(http.MethodPost, "/orders/reserve", bytes.NewBufferString(`{
		"account_id": "acct_123",
		"order_id": "ord_123",
		"requested_cents": 1900
	}`))

	handler.ReserveOrder(recorder, request)

	if recorder.Code != http.StatusCreated {
		t.Fatalf("status = %d, want %d; body=%s", recorder.Code, http.StatusCreated, recorder.Body.String())
	}
	if ledger.reservedCents != 1900 {
		t.Fatalf("reserved cents = %d, want 1900", ledger.reservedCents)
	}
}

func TestReserveOrderRejectsInsufficientBalanceWithoutReserving(t *testing.T) {
	ledger := &fakeLedger{}
	handler := Handler{
		Wallet: fakeWallet{balanceCents: 1200},
		Ledger: ledger,
	}

	recorder := httptest.NewRecorder()
	request := httptest.NewRequest(http.MethodPost, "/orders/reserve", bytes.NewBufferString(`{
		"account_id": "acct_123",
		"order_id": "ord_123",
		"requested_cents": 1900
	}`))

	handler.ReserveOrder(recorder, request)

	if recorder.Code != http.StatusConflict {
		t.Fatalf("status = %d, want %d; body=%s", recorder.Code, http.StatusConflict, recorder.Body.String())
	}
	if ledger.called {
		t.Fatal("ledger was called for an insufficient-balance request")
	}
}

func TestReserveOrderRejectsNegativeRequestBeforePolicyCall(t *testing.T) {
	ledger := &fakeLedger{}
	handler := Handler{
		Wallet: fakeWallet{balanceCents: 1200},
		Ledger: ledger,
	}

	recorder := httptest.NewRecorder()
	request := httptest.NewRequest(http.MethodPost, "/orders/reserve", bytes.NewBufferString(`{
		"account_id": "acct_123",
		"order_id": "ord_123",
		"requested_cents": -1
	}`))

	handler.ReserveOrder(recorder, request)

	if recorder.Code != http.StatusBadRequest {
		t.Fatalf("status = %d, want %d; body=%s", recorder.Code, http.StatusBadRequest, recorder.Body.String())
	}
	if ledger.called {
		t.Fatal("ledger was called for an invalid request")
	}
}

func TestReserveOrderHandlesWalletFailureWithoutReserving(t *testing.T) {
	ledger := &fakeLedger{}
	handler := Handler{
		Wallet: fakeWallet{err: errors.New("wallet down")},
		Ledger: ledger,
	}

	recorder := httptest.NewRecorder()
	request := httptest.NewRequest(http.MethodPost, "/orders/reserve", bytes.NewBufferString(`{
		"account_id": "acct_123",
		"order_id": "ord_123",
		"requested_cents": 1900
	}`))

	handler.ReserveOrder(recorder, request)

	if recorder.Code != http.StatusBadGateway {
		t.Fatalf("status = %d, want %d; body=%s", recorder.Code, http.StatusBadGateway, recorder.Body.String())
	}
	if ledger.called {
		t.Fatal("ledger was called after a wallet failure")
	}
}

type fakeWallet struct {
	balanceCents int64
	err          error
}

func (w fakeWallet) AvailableCents(context.Context, string) (int64, error) {
	if w.err != nil {
		return 0, w.err
	}
	return w.balanceCents, nil
}

type fakeLedger struct {
	called        bool
	reservedCents int64
}

func (l *fakeLedger) Reserve(_ context.Context, _ string, _ string, cents int64) error {
	l.called = true
	l.reservedCents = cents
	return nil
}
