package webapp

import (
	"context"
	"encoding/json"
	"errors"
	"io"
	"net/http"
	"strings"

	orderpolicy "example.com/orderpolicy"
)

type Wallet interface {
	AvailableCents(ctx context.Context, accountID string) (int64, error)
}

type Ledger interface {
	Reserve(ctx context.Context, accountID string, orderID string, cents int64) error
}

type Handler struct {
	Wallet Wallet
	Ledger Ledger
}

type reserveRequest struct {
	AccountID      string `json:"account_id"`
	OrderID        string `json:"order_id"`
	RequestedCents int64  `json:"requested_cents"`
}

type reserveResponse struct {
	Accepted       bool  `json:"accepted"`
	ApprovedCents  int64 `json:"approved_cents"`
	AvailableCents int64 `json:"available_cents,omitempty"`
}

var (
	ErrBadRequest          = errors.New("bad request")
	ErrInsufficientBalance = errors.New("insufficient balance")
)

func (h Handler) ReserveOrder(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		w.Header().Set("Allow", http.MethodPost)
		http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		return
	}
	if h.Wallet == nil || h.Ledger == nil {
		http.Error(w, "server not configured", http.StatusInternalServerError)
		return
	}

	request, err := decodeReserveRequest(r)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	balanceCents, err := h.Wallet.AvailableCents(r.Context(), request.AccountID)
	if err != nil {
		http.Error(w, "wallet unavailable", http.StatusBadGateway)
		return
	}
	if balanceCents < 0 {
		http.Error(w, "wallet invariant violated", http.StatusInternalServerError)
		return
	}

	approvedCents := orderpolicy.ApprovedReserveCents(balanceCents, request.RequestedCents)
	if approvedCents != request.RequestedCents {
		writeJSON(w, http.StatusConflict, reserveResponse{
			Accepted:       false,
			ApprovedCents:  approvedCents,
			AvailableCents: balanceCents,
		})
		return
	}

	if err := h.Ledger.Reserve(r.Context(), request.AccountID, request.OrderID, approvedCents); err != nil {
		http.Error(w, "ledger unavailable", http.StatusBadGateway)
		return
	}

	writeJSON(w, http.StatusCreated, reserveResponse{
		Accepted:      true,
		ApprovedCents: approvedCents,
	})
}

func decodeReserveRequest(r *http.Request) (reserveRequest, error) {
	defer r.Body.Close()

	var request reserveRequest
	decoder := json.NewDecoder(r.Body)
	decoder.DisallowUnknownFields()
	if err := decoder.Decode(&request); err != nil {
		return reserveRequest{}, ErrBadRequest
	}
	var trailing any
	if err := decoder.Decode(&trailing); err != io.EOF {
		return reserveRequest{}, ErrBadRequest
	}

	request.AccountID = strings.TrimSpace(request.AccountID)
	request.OrderID = strings.TrimSpace(request.OrderID)
	if request.AccountID == "" || request.OrderID == "" || request.RequestedCents < 0 {
		return reserveRequest{}, ErrBadRequest
	}
	return request, nil
}

func writeJSON(w http.ResponseWriter, status int, value any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(value)
}
