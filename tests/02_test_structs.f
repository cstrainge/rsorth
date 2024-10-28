
( Define our structures... )
# bar x y z ;

# foo
    a
    b
    c -> bar.new
;


( Create a variable to hold an instance of foo. )
variable fp


( The raw value we get from variable creation. )
"New variable:            " . fp @ .cr


( Create an instance of the struct foo and store it in our variable. )
foo.new fp !
"Uninitialized struct:    " . fp @ .cr


( Assign some values to the first two fields. )
1024           fp foo.a!!
"Hello world!" fp foo.b!!


( Now fill out the rest of foo.c's bar fields. )
150 fp foo.c@@ bar.x!
250 fp foo.c@@ bar.y!
350 fp foo.c@@ bar.z!

( Finally print the whole thing. )
"Initialized struct(s):   " . fp @ .cr
