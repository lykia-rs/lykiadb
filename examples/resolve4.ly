var $a = "global";
{
  fun showA() {
    print($a);
  }

  var $a = "block";
  
  fun showB() {
    print($a);
  }

  //
  showA();
  showB();
  //
  $a = "test";
  //
  showA();
  showB();
}