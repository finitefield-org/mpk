namespace Fuzz;
public static class Seed
{
    public static int F(int x) { return unchecked(x + 1); }
}
