var $a = "global";
{
  fun showA() {
    print($a);
  }

  showA();
  var $a = "block";
  showA();
  fun showB() {
    print($a);
  }
  showB();
}