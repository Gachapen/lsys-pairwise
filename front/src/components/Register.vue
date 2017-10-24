<template lang="pug">
  .register
    section.user-info(v-if='!token')
      h2 Your information
      p
        label(for='age')
          span Age:
        input(id='age' type='number' v-model='age')
      p
        label(for='gender')
          span Gender:
        select(id='gender' v-model='gender')
          option(value='' disabled selected) Select
          option(value='female') Female
          option(value='male') Male
          option(value='other') Other
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
import { post } from 'axios'
import { API_BASE } from '../config'

export default {
  data () {
    return {
      age: undefined,
      gender: undefined,
      token: null,
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
      })
        .then(response => {
          this.token = response.data.token
        })
        .catch(error => console.error('Failed registering user', error))
    },
  },
}
</script>

<style scoped lang="sass">
.register
  overflow: auto
  width: 100%
  height: 100%
  margin: 0
  padding: 0
  background: #282828
  color: white

section
  width: 500px
  margin: 0 auto
  text-align: left

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
</style>
