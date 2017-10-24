<template>
  <div id="app">
    <router-view/>
  </div>
</template>

<script>
import VueRouter from 'vue-router'
import Register from './components/Register'
import Task from './components/Task'

const router = new VueRouter({
  routes: [
    {
      path: '/',
      component: Register,
    },
    {
      path: '/task/:token',
      component: Task,
      props: true,
    },
  ],
})

export default {
  name: 'app',
  router,
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
