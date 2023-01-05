## TODO

### Immediate/Before next chapter
- [x] Fix the error reporting to run tests from the main source.

### Revisit in the future after I finish the book.
- [ ] Getter has a bug in it. It returns a clone. `a.b.c = 42` would end up not as expected. 
- [ ] Above problem can be solved by a garbage collection, I guess.
- [ ] Remove cloning in `Function.bind`. Not removing could lead to bugs. Maybe not! Value has Rc in it. 
- [ ] Pass `test/operator/equals_method.lox`. 
- [ ] Revisit visit pattern!
- [ ] Print integers without the `.0` suffix.
- [ ] Too many clones
    - [ ] In environment