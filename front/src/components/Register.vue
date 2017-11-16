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
          label(for='education') Education level: #[br] (in progress or completed)
        .input-field
          select(
            id='education'
            name='education'
            v-model='education'
            v-validate="'required'"
            :class='{ "danger": errors.has("education") }'
          )
            option(value='' disabled) Select
            option(value='none') None
            option(value='primary') Primary
            option(value='secondary') Secondary
            option(value='bachelor') Bachelor
            option(value='master') Master
            option(value='doctoral') Doctoral
          .help.danger(v-show='errors.has("education")') {{ errors.first('education') }}
      .row
        .input-label
          label(for='occupation') Occupation / studies:
        .input-field
          select(
            id='occupation'
            name='occupation'
            v-model='occupation'
            v-validate="'required'"
            :class='{ "danger": errors.has("occupation") }'
          )
            option(value='' disabled) Select
            option(value='agricultural_forestry_and_fishery') Agricultural, forestry and fishery
            option(value='armed_forces') Armed forces
            option(value='business_and_administration') Business and administration
            option(value='clerical_support') Clerical support
            option(value='craft_and_related_trades') Craft and related trades
            option(value='creative_artist') Creative artist
            option(value='cultural') Cultural
            option(value='health_medicine') Health/Medicine
            option(value='information_and_communication_technology') Information and communication technology
            option(value='legal') Legal
            option(value='management') Management
            option(value='manual_labor') Manual labor
            option(value='plant_and_machine_operation_and_assembly') Plant and machine operation and assembly
            option(value='science_and_engineering') Science and engineering
            option(value='service_and_sales') Service and sales
            option(value='social_work') Social work
            option(value='sport') Sport
            option(value='teaching') Teaching
            option(value='other') Other
          .help.danger(v-show='errors.has("occupation")') {{ errors.first('occupation') }}
      .row(v-if='occupation === "other"')
        .input-label
          label(for='other_occupation') Specify occupation / studies:
        .input-field
          input(
            id='other_occupation'
            name='other_occupation'
            type='text'
            v-model='other_occupation'
            v-validate="'required'"
            :class='{ "danger": errors.has("other_occupation") }'
          )
          .help.danger(v-show='errors.has("other_occupation")') {{ errors.first('other_occupation') }}
      //- .row
      //-   .input-label
      //-     label(for='task') Task:
      //-   .input-field
      //-     select(
      //-       id='task'
      //-       name='task'
      //-       v-model='task'
      //-       v-validate="'required'"
      //-       :class='{ "danger": errors.has("task") }'
      //-     )
      //-       option(value='' disabled) Select
      //-       option(v-for='task of tasks' :value='task') {{ task | capitalize }}
      //-     .help.danger(v-show='errors.has("task")') {{ errors.first('task') }}
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
      default: 'experiment2',
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
      education: '',
      occupation: '',
      other_occupation: undefined,
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

          let occupation = this.occupation
          if (occupation === 'other') {
            occupation = {
              'other': this.other_occupation,
            }
          }

          post(`${API_BASE}/user`, {
            age: parseInt(this.age, 10),
            gender: this.gender,
            education: this.education,
            occupation: occupation,
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
  width: 200px
  text-align: right
</style>
