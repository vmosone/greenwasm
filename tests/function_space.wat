(module
  (type $t0 (func (param i32) (result i32)))
  (type $t1 (func (param f32)))
  (type $t2 (func))
  (import "foo" "bar" (func $foo.bar (type $t1)))
  (func $f1 (type $t2)
    (nop))
  (table $T0 0 1 anyfunc)
  (memory $M0 1 1)
  (start 1))
