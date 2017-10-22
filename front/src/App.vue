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
    Pairwise,
  },
}
</script>

<style lang="sass">
html, body, #app
  width: 100%
  height: 100%
  margin: 0
  padding: 0

html
  box-sizing: border-box
*, *:before, *:after
  box-sizing: inherit

#app
  font-family: 'Avenir', Helvetica, Arial, sans-serif
  -webkit-font-smoothing: antialiased
  -moz-osx-font-smoothing: grayscale
  text-align: center
  color: #2c3e50
</style>
