<template>
  <div id="app">
    <Pairwise/>
  </div>
</template>

<script>
import Pairwise from './components/Pairwise'

export default {
  name: 'app',
  components: {
    Pairwise
  }
}
</script>

<style>
#app {
  font-family: 'Avenir', Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
  margin-top: 60px;
}
</style>
