const rust = import('./pkg');

const f = (x) => {
    console.log(x);
}

rust
    .then(m => m.run(f))
    .catch(console.error)
;
