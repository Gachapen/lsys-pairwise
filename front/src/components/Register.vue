<template lang="pug">
  .register
    header
      h2 Register
    section.user-info
      .row
        .input-label
          label(for='age') Age:
        .input-field
          input(
            id='age'
            type='number'
            name='age'
            v-model='age'
            v-validate="'required|numeric|min_value:1|max_value:100'"
            :class='{ "danger": errors.has("age") }'
          )
          .help.danger(v-show='errors.has("age")') {{ errors.first('age') }}
      .row
        .input-label
          label(for='gender') Gender:
        .input-field
          select(
            id='gender'
            name='gender'
            v-model='gender'
            v-validate="'required'"
            :class='{ "danger": errors.has("gender") }'
          )
            option(value='' disabled) Select
            option(value='female') Female
            option(value='male') Male
            option(value='other') Other
          .help.danger(v-show='errors.has("gender")') {{ errors.first('gender') }}
      .row
        .input-label
          label(for='task') Task:
        .input-field
          select(
            id='task'
            name='task'
            v-model='task'
            v-validate="'required'"
            :class='{ "danger": errors.has("task") }'
          )
            option(value='' disabled) Select
            option(v-for='task of tasks' :value='task') {{ task | capitalize }}
          .help.danger(v-show='errors.has("task")') {{ errors.first('task') }}
    section.questionaire
      section
        likert-scale(
          statement='How often do you work with plants?'
          details='Including all types of interactions with plants, such as gardening, household plants, photography, etc.'
          scale='frequency'
          name='work-with-plants'
          v-model='plantWork'
        )
      section
        likert-scale(
          statement='How much do you like plants in general?'
          scale='like'
          name='like-plants'
          v-model='plantLike'
        )
      section
        likert-scale(
          statement='How often do you play video games?'
          scale='frequency'
          name='video-game'
          v-model='gameFrequency'
        )
    section.submit
      p
        button(@click='register()') Register
</template>

<script>
import { post, get } from 'axios'
import { detect } from 'detect-browser'
import { API_BASE } from '../config'
import LikertScale from './LikertScale'

export default {
  components: {
    LikertScale,
  },
  inject: ['$validator'],
  props: {
    initialTask: {
      type: String,
      required: false,
      default: '',
    },
    from: {
      type: String,
      required: false,
    },
    source: {
      type: String,
      required: false,
      default: 'url',
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
      this.$validator.validateAll().then(result => {
        if (result) {
          let preQuestionnaire = null
          if (this.plantWork !== undefined || this.plantLike !== undefined || this.gameFrequency !== undefined) {
            preQuestionnaire = {
              plant_work: this.plantWork,
              plant_like: this.plantLike,
              video_game: this.gameFrequency,
            }
          }

          let browserInfo = null
          const browser = detect()
          if (browser) {
            browserInfo = {
              name: browser.name,
              version: browser.version,
            }
          }

          post(`${API_BASE}/user`, {
            age: parseInt(this.age, 10),
            gender: this.gender,
            task: this.task,
            pre_questionnaire: preQuestionnaire,
            from: this.from,
            source: this.source,
            browser: browserInfo,
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
        }
      })
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
.input-label
  width: 80px
  text-align: right
</style>
