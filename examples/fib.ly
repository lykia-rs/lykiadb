// Define recursive Fibonacci function
fun fib(n) {
  if (n < 2) return n;
  return fib(n - 2) + fib(n - 1);
}

// Calculate the total execution time
var start = clock();
print(fib(35) == 9227465);
print("elapsed:");
print(clock() - start);