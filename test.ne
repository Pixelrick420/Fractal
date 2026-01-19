!start
!import "lib\sample_lib.fr";
# this is a comment


:int a = 4;
:float b = 5.0;

:array<int,5> arr = [1, 2, 3, 4, 5];
:array<char,10> string = "hello 1234";
:list<int> nums = [1, 2, 3];

!if(a > b){
    a = (a + 1);
}
!else {
    b = (b + (float) 1);
}

append(nums, a);
append(arr, (int) b);

:int c = pop(arr);


!for (:int i, 0, n, 1) {     # variable, start, stop, step
    print("{}, {}, {}", a, b, c);
    # {} acts as a placeholder for a simple variable(int float char and bool each must have a string form that can be printed).
    # There is no native method for printing iterables or complex variables (let user make those)
}

!while (a > b) {
    a -= 1;
    print("Hello");
}

!func hello(:int a, :float b) -> :bool{
    !return (a > b);
}

:struct<STRUCTNAME> {
    :int a;
    :int b;
    :array<int,10> arr;
    :struct<STRUCTNAME> next; # could be self reference
}

!end