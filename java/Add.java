public class Add {

  public static int subtract(int a, int b) {
    return add(a, -b);
  }

  public static int add(int a, int b) {
    return a + b;
  }

  public static double doubleAddHalf(double a) {
    double c = -0.5;
    return a + c;
  }

  public static double doubleAdd(double a, double b) {
    return a + b;
  }

  public static float floatAddHalf(float f) {
    float c = -0.5f;
    return f + c;
  }

  public static long longAddConst(long l) {
    long c = -9l;
    return l + c;
  }

  public static int intAddConst(int i) {
    return i + 1_000_000;
  }

  public static int addMany(int a, int b, int c, int d, int f, int e) {
    return a + b + c+ d+ e+ f;
  }
}

