namespace Business;
public readonly struct State {
    public readonly ulong Version;
    public readonly int Balance;
    public State(ulong version, int balance) { Version = version; Balance = balance; }
}
public readonly struct Command {
    public readonly ulong Expected;
    public readonly int Amount;
    public Command(ulong expected, int amount) { Expected = expected; Amount = amount; }
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
public enum DomainError { VersionConflict = 0, VersionExhausted = 1, Negative = 2 }
public readonly struct ApplyResult {
    public readonly Tag Tag;
    public readonly Change Value;
    public readonly DomainError Error;
    public ApplyResult(Tag tag, Change value, DomainError error) { Tag = tag; Value = value; Error = error; }
}
public static class Entry {
    public static ApplyResult Apply(State state, Command command, Context context) {
        Change inactive = new Change(state, new Event[0], new Response(state.Balance));
        if (state.Version != command.Expected) return new ApplyResult(Tag.Error, inactive, DomainError.VersionConflict);
        if (state.Version == 18446744073709551615UL) return new ApplyResult(Tag.Error, inactive, DomainError.VersionExhausted);
        if (command.Amount < 0) return new ApplyResult(Tag.Error, inactive, DomainError.Negative);
        State next = new State(state.Version + 1UL, state.Balance + command.Amount);
        Event[] events = new Event[] { new Event(command.Amount, context.Effective), new Event(0, context.Effective) };
        return new ApplyResult(Tag.Ok, new Change(next, events, new Response(next.Balance)), DomainError.VersionConflict);
    }
    // The pinned CLR harness observes one integer at a time from the actual Apply.
    public static int Run(int n, int[] a, string s) {
        ulong version = a[0] < 0 ? 18446744073709551615UL : (ulong)a[0];
        ulong expected = a[1] < 0 ? 18446744073709551615UL : (ulong)a[1];
        State state = new State(version, a[3]);
        ApplyResult result = Apply(state, new Command(expected, a[2]), new Context((long)a[4]));
        if (n == 0) return result.Tag == Tag.Ok ? 0 : 1;
        if (n == 1) return result.Error == DomainError.VersionConflict ? 0 : result.Error == DomainError.VersionExhausted ? 1 : 2;
        if (n == 8) return state.Balance;
        if (n == 9) return state.Version == 18446744073709551615UL ? -1 : (int)state.Version;
        if (result.Tag == Tag.Error) return -1;
        if (n == 2) return (int)result.Value.State.Version;
        if (n == 3) return result.Value.State.Balance;
        if (n == 4) return result.Value.Events.Length;
        if (n == 5) return result.Value.Events[0].Amount;
        if (n == 6) return (int)result.Value.Events[0].Effective;
        if (n == 7) return result.Value.Response.Balance;
        return result.Value.Events[1].Amount;
    }
}
