fun makeCounter() {
    var i = 0;
    fun count() {
        i = i + 1;
        print(i);
    }

    return count;
}
var start = clock();
var counter = makeCounter();
counter(); // "1".
counter(); // "2".
print(clock() - start);