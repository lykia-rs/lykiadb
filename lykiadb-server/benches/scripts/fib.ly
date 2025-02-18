// Define recursive Fibonacci function
function $fib($n: dtype::num) {
  if ($n < 2) return $n;
  return $fib($n - 2) + $fib($n - 1);
};

$fib(15);