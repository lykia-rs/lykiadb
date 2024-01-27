function helloWorld ($message) {
    print("Hello world!", $message);
    {
        {
            return "and returning from here.";
            {
                print("inner");
                print("inner");
                print("inner");
            }
            print("outer");
            print("outer");
            print("outer");
        }
    }
};

for (var $i = 0; $i < 10; $i = $i + 1) {
    print(helloWorld("My name is Lykia."));
}
