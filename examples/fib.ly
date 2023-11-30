// Define recursive Fibonacci function
fun fib($n) {
  if ($n < 2) return $n;
  return fib($n - 2) + fib($n - 1);
};

var $start_ly = clock();
print(fib(35) == 9227465);
print("elapsed (user defined):", clock() - $start_ly);

var $start_rs = clock();
print(fib_nat(35) == 9227465);
print("elapsed (native):", clock() - $start_rs);