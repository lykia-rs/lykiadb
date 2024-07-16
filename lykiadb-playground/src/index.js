import loadWasm from '../../lykiadb-lang/src/lib.rs';
loadWasm().then(result => {
  console.log(result.instance.exports)
});