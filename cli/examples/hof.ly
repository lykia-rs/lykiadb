function makeCounter() {
    var $i = 0;
    function count() {
        $i = $i + 1;
        print($i);
    };

    return count;
};
var $count = makeCounter();
$count();
$count();