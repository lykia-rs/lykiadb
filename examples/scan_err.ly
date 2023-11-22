// Define recursive Fibonacci function
fun fib($n) {
  if ($n < 2) return $n;
  return fib($n - 2) + fib($n - 1);
}

117E