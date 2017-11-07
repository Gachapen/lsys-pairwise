<template lang="pug">
  .register
    section.info
      h2 Information
      p.
        During this experiment you will be asked to rate how much more pleasing a plant is
        compared to another. If they are equally pleasing, select 0. If one plant is more
        pleasing than the other, select 1, 2 or 3 on the same side as the plant to indicate how
        much more pleasing it is.
    section.user-info(v-if='!token')
      h2 Register
      p
        label(for='age')
          span Age:
        input(id='age' type='number' v-model='age')
      p
        label(for='gender')
          span Gender:
        select(id='gender' v-model='gender')
          option(value='' disabled) Select
          option(value='female') Female
          option(value='male') Male
          option(value='other') Other
      p
        label(for='task')
          span Task:
        select(id='task' v-model='task')
          option(value='' disabled) Select
          option(v-for='task of tasks' :value='task') {{ task | capitalize }}
      p
        button(@click='register()') Register
    section.begin(v-if='token')
      h2 Begin task
      p
        label(for='token')
          span Token:
        output(id='token') {{ token }}
      p
        router-link(:to='taskLink' role='button') Begin
</template>

<script>
import { post, get } from 'axios'
import { API_BASE } from '../config'

export default {
  data () {
    return {
      age: undefined,
      gender: '',
      token: null,
      task: '',
      tasks: [],
    }
  },
  computed: {
    taskLink () {
      return `/task/${this.token}`
    },
  },
  methods: {
    register () {
      post(`${API_BASE}/user`, {
        age: parseInt(this.age, 10),
        gender: this.gender,
        task: this.task,
      })
        .then(response => {
          this.token = response.data.token
        })
        .catch(error => console.error('Failed registering user', error))
    },
  },
  created () {
    get(`${API_BASE}/task`)
      .then(response => {
        this.tasks = response.data
      })
      .catch(error => console.error('Failed retrieving tasks', error))
  },
}
</script>

<style scoped lang="sass">
.register
  overflow: auto
  width: 800px
  height: 100%
  margin: 0 auto
  padding: 0
  color: white
  text-align: left

section
  >h2
    text-align: center

section.user-info, section.begin
  margin: 0 auto

  label
    margin-right: 5px
    >span
      display: inline-block
      width: 120px
      text-align: right

  button, a[role="button"]
    margin-left: 125px
    padding: 8px
    font-size: 12pt
    background: #666666
    border: 0
    border-radius: 3px
    color: white
    text-decoration: none

section.user-info
  width: 400px

section.begin
  width: 500px
</style>
