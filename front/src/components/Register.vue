<template lang="pug">
  .register
    header
      h2 Register
    section.user-info
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
    section.submit
      p
        button(@click='register()') Register
</template>

<script>
import { post, get } from 'axios'
import { API_BASE } from '../config'

export default {
  props: {
    initialTask: {
      type: String,
      required: false,
      default: '',
    },
  },
  data () {
    return {
      age: undefined,
      gender: '',
      task: this.initialTask,
      tasks: [],
    }
  },
  computed: {
  },
  methods: {
    register () {
      post(`${API_BASE}/user`, {
        age: parseInt(this.age, 10),
        gender: this.gender,
        task: this.task,
      })
        .then(response => {
          this.$router.push({
            name: 'intro',
            params: {
              token: response.data.token,
            },
          })
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
section.user-info
  >p
    >label
      margin-right: 10px
      >span
        display: inline-block
        width: 80px
        text-align: right
</style>
