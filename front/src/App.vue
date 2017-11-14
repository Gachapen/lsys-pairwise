<template>
  <div id="app">
    <router-view/>
  </div>
</template>

<script>
import VueRouter from 'vue-router'
import Consent from './components/Consent'
import Register from './components/Register'
import Intro from './components/Intro'
import Task from './components/Task'
import Result from './components/Result'
import ThankYou from './components/ThankYou'

const router = new VueRouter({
  routes: [
    {
      path: '/',
      component: Consent,
    },
    {
      path: '/register',
      component: Register,
      props: route => ({
        initialTask: route.query.task,
        from: route.query.from,
        source: route.query.source,
      }),
    },
    {
      name: 'intro',
      path: '/intro/:token',
      component: Intro,
      props: true,
    },
    {
      name: 'task',
      path: '/task/:token',
      component: Task,
      props: true,
    },
    {
      name: 'result',
      path: '/result/:token',
      component: Result,
      props: route => ({
        questions: !route.query['no-questions'],
        token: route.params.token,
      }),
    },
    {
      name: 'thanks',
      path: '/thanks/:token',
      component: ThankYou,
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
$fa-font-path: '~font-awesome/fonts'
@import '~font-awesome/scss/font-awesome'

html, body, #app
  width: 100%
  height: 100%
  margin: 0
  padding: 0
  background: #282828
  color: white

html
  box-sizing: border-box
*, *:before, *:after
  box-sizing: inherit

a
  color: white

button, a[role="button"]
  padding: 8px
  font-size: 12pt
  background: #666666
  border: 0
  border-radius: 3px
  color: white
  text-decoration: none

  &:hover:not(.disabled):not(:disabled)
    cursor: pointer

  &:disabled, &.disabled
    background: #363636
    color: #777

.disabled
  pointer-events: none

section
  max-width: 800px
  margin: 0 auto
  padding: 0 10px

  >h2
    text-align: center
  >p
    text-align: left

section.questionnaire
  >section
    margin-top: 40px
    margin-bottom: 34px

    >h4
      text-align: left

    textarea
      width: 100%
      height: 100px

.row
  display: table
  margin: 20px 0

.input-label, .input-field
  float: left

.input-label
  >label
    display: inline-block
    margin-right: 10px

.input-field
  text-align: left
  >.help
    margin-top: 5px

.danger.help
  color: #f66

.cancel a
  opacity: 0.7

#app
  font-family: 'Avenir', Helvetica, Arial, sans-serif
  -webkit-font-smoothing: antialiased
  -moz-osx-font-smoothing: grayscale
  text-align: center

  >*
    overflow: auto

    >header
      >h2
        text-align: center
</style>
