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
    section.questionaire
      section
        likert-scale(
          statement='How often do you work with plants?'
          details='Including all types of interactions with plants, such as gardening, household plants, photography, etc.'
          scale='frequency'
          v-model='plantWork'
        )
      section
        likert-scale(
          statement='How much do you like plants in general?'
          scale='like'
          v-model='plantLike'
        )
      section
        likert-scale(
          statement='How often do you play video games?'
          scale='frequency'
          v-model='gameFrequency'
        )
    section.submit
      p
        button(@click='register()') Register
</template>

<script>
import { post, get } from 'axios'
import { API_BASE } from '../config'
import LikertScale from './LikertScale'

export default {
  components: {
    LikertScale,
  },
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
      plantWork: undefined,
      plantLike: undefined,
      gameFrequency: undefined,
    }
  },
  computed: {
  },
  methods: {
    register () {
      let preQuestionnaire = null
      if (this.plantWork !== undefined || this.plantLike !== undefined || this.gameFrequency !== undefined) {
        preQuestionnaire = {
          plant_work: this.plantWork,
          plant_like: this.plantLike,
          video_game: this.gameFrequency,
        }
      }

      post(`${API_BASE}/user`, {
        age: parseInt(this.age, 10),
        gender: this.gender,
        task: this.task,
        pre_questionnaire: preQuestionnaire,
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
