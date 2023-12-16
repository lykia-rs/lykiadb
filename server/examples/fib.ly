// Define recursive Fibonacci function
fun fib($n) {
  if ($n < 2) return $n;
  return fib($n - 2) + fib($n - 1);
};

var $start_ly = Time.clock();
print(fib(35) == 9227465);
print("elapsed (user defined):", Time.clock() - $start_ly);

var $start_rs = Time.clock();
print(Benchmark.fib(35) == 9227465);
print("elapsed (native):", Time.clock() - $start_rs);