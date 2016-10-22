<template>
  <div id="app">
    <hello></hello>
  </div>
</template>

<script>
import Hello from './components/Hello'
// import store from 'src/store'

export default {
  components: {
    Hello
  }
}
</script>

<style>
html, body {
  width:  100%;
  height: 100%;
  margin: 0px;
  padding: 0px;
  overflow: hidden;
}

#app {
  color: #2c3e50;
  font-family: Source Sans Pro, Helvetica, sans-serif;
}
</style>
