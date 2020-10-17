public class Add {

  private int[] args;

  public Add(int a, int b) {
    this.args = new int[2];
    args[0] = a;
    args[1] = b;
  }
//
//  public static int subtract(int a, int b) {
//    return add(a, -b);
//  }
//
//  public static int add(int a, int b) {
//    return a + b;
//  }
//
//  public static double doubleAddHalf(double a) {
//    double c = -0.5;
//    return a + c;
//  }
//
//  public static double doubleAdd(double a, double b) {
//    return a + b;
//  }
//
//  public static float floatAddHalf(float f) {
//    float c = -0.5f;
//    return f + c;
//  }
//
//  public static long longAddConst(long l) {
//    long c = -9l;
//    return l + c;
//  }
//
//  public static int intAddConst(int i) {
//    return i + 1_000_000;
//  }
//
//  public static int addMany(int a, int b, int c, int d, int f, int e) {
//    return a + b + c+ d+ e+ f;
//  }
//
//  public static int doStuff() {
//    int i = 0;
//    while (i < 10) {
//      i++;
//    }
//    return i;
//  }

  public int result() {
    return this.args[0] + this.args[1];
  }

  public static int main() {
    Add a = new Add(5, 6);
    return a.result();
  }








}

