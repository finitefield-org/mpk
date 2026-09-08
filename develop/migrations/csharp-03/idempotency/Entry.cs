namespace Business;
public readonly struct State {
    public readonly ulong Version;
    public readonly int Balance;
    public readonly Processed[] History;
    public State(ulong version, int balance, Processed[] history) { Version = version; Balance = balance; History = history; }
}
public readonly struct Command {
    public readonly ulong Expected;
    public readonly int Amount;
    public readonly int Key;
    public readonly Presence Note;
    public Command(ulong expected, int amount, int key, Presence note) { Expected = expected; Amount = amount; Key = key; Note = note; }
}
public enum PresenceTag { Missing = 0, Null = 1, Value = 2 }
public readonly struct Presence {
    public readonly PresenceTag Tag;
    public readonly int Value;
    public Presence(PresenceTag tag, int value) { Tag = tag; Value = value; }
}
public readonly struct Processed {
    public readonly int Key;
    public readonly Command Command;
    public readonly Context Context;
    public readonly Response Response;
    public Processed(int key, Command command, Context context, Response response) { Key = key; Command = command; Context = context; Response = response; }
}
public readonly struct Context {
    public readonly long Effective;
    public Context(long effective) { Effective = effective; }
}
public readonly struct Event {
    public readonly int Amount;
    public readonly long Effective;
    public Event(int amount, long effective) { Amount = amount; Effective = effective; }
}
public readonly struct Response {
    public readonly int Balance;
    public Response(int balance) { Balance = balance; }
}
public readonly struct Change {
    public readonly State State;
    public readonly Event[] Events;
    public readonly Response Response;
    public Change(State state, Event[] events, Response response) { State = state; Events = events; Response = response; }
}
public enum Tag { Ok = 0, Error = 1 }
public enum DomainError { VersionConflict = 0, VersionExhausted = 1, Negative = 2, IdempotencyConflict = 3, HistoryCapacity = 4 }
public readonly struct ApplyResult {
    public readonly Tag Tag;
    public readonly Change Value;
    public readonly DomainError Error;
    public ApplyResult(Tag tag, Change value, DomainError error) { Tag = tag; Value = value; Error = error; }
}
public static class Entry {
    public static bool SameSnapshot(Command a, Context x, Command b, Context y) {
        return a.Expected == b.Expected && a.Amount == b.Amount && a.Key == b.Key
            && a.Note.Tag == b.Note.Tag && a.Note.Value == b.Note.Value && x.Effective == y.Effective;
    }
    public static ApplyResult Apply(State state, Command command, Context context) {
        Change inactive = new Change(state, new Event[0], new Response(state.Balance));
        for (int i = 0; i < state.History.Length; i++) {
            Processed old = state.History[i];
            if (old.Key == command.Key) {
                if (SameSnapshot(old.Command, old.Context, command, context))
                    return new ApplyResult(Tag.Ok, new Change(state, new Event[0], old.Response), DomainError.VersionConflict);
                return new ApplyResult(Tag.Error, inactive, DomainError.IdempotencyConflict);
            }
        }
        if (state.Version != command.Expected) return new ApplyResult(Tag.Error, inactive, DomainError.VersionConflict);
        if (state.History.Length == 4096) return new ApplyResult(Tag.Error, inactive, DomainError.HistoryCapacity);
        if (state.Version == 18446744073709551615UL) return new ApplyResult(Tag.Error, inactive, DomainError.VersionExhausted);
        if (command.Amount < 0) return new ApplyResult(Tag.Error, inactive, DomainError.Negative);
        Response response = new Response(state.Balance + command.Amount);
        Processed[] history = new Processed[state.History.Length + 1];
        for (int i = 0; i < state.History.Length; i++) history[i] = state.History[i];
        history[state.History.Length] = new Processed(command.Key, command, context, response);
        State next = new State(state.Version + 1UL, state.Balance + command.Amount, history);
        Event[] events = new Event[] { new Event(command.Amount, context.Effective), new Event(0, context.Effective) };
        return new ApplyResult(Tag.Ok, new Change(next, events, new Response(next.Balance)), DomainError.VersionConflict);
    }
    // Integer observations of actual CLR Apply, not a proof or production oracle.
    public static int Run(int n, int[] a, string s) {
        ulong version = a[0] < 0 ? 18446744073709551615UL : (ulong)a[0];
        ulong expected = a[1] < 0 ? 18446744073709551615UL : (ulong)a[1];
        ulong oldExpected = a[10] < 0 ? 18446744073709551615UL : (ulong)a[10];
        PresenceTag tag = a[6] == 0 ? PresenceTag.Missing : a[6] == 1 ? PresenceTag.Null : PresenceTag.Value;
        PresenceTag oldTag = a[13] == 0 ? PresenceTag.Missing : a[13] == 1 ? PresenceTag.Null : PresenceTag.Value;
        Processed[] history = new Processed[a[8]];
        for (int i = 0; i < history.Length; i++) history[i] = new Processed(a[9] + i,
            new Command(oldExpected, a[11], a[9] + i, new Presence(oldTag, a[14])), new Context((long)a[12]), new Response(77 + i));
        State state = new State(version, a[3], history);
        ApplyResult result = Apply(state, new Command(expected, a[2], a[5], new Presence(tag, a[7])), new Context((long)a[4]));
        if (n == 0) return result.Tag == Tag.Ok ? 0 : result.Error == DomainError.IdempotencyConflict ? 1
            : result.Error == DomainError.VersionConflict ? 2 : result.Error == DomainError.HistoryCapacity ? 3
            : result.Error == DomainError.VersionExhausted ? 4 : 5;
        if (n == 1) return result.Value.State.Version == 18446744073709551615UL ? -1 : (int)result.Value.State.Version;
        if (n == 2) return result.Value.State.Balance;
        if (n == 3) return result.Value.Events.Length;
        if (n == 4) return result.Value.Response.Balance;
        if (n == 5) return result.Value.State.History.Length;
        if (n == 6) return result.Value.State.History.Length == 0 ? -1 : result.Value.State.History[0].Key;
        if (n == 7) return result.Value.State.History.Length == 0 ? -1 : result.Value.State.History[result.Value.State.History.Length - 1].Key;
        if (n == 8) return state.Balance;
        if (n == 9) return state.History.Length;
        if (n == 10) return result.Value.Events.Length == 0 ? -1 : result.Value.Events[0].Amount;
        if (n == 11) return result.Value.Events.Length == 0 ? -1 : result.Value.Events[1].Amount;
        return history.Length == 0 ? -1 : history[0].Response.Balance;
    }
}
