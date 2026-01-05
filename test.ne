!start

a = 1;
b = 1;

!for (i = 1; i < 10; i = (i + 1)){
    t = a;
    a = b;
    b = (t + b);
}

!exit b;

!end