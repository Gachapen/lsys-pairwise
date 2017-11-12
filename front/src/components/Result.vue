<template lang="pug">
  .result
    section.metric.pleasing
      h2 Ranking of plants based on your answers
      p.details(v-if='questions') Please submit the post-questionaire below
      template(v-if='pleasing_ranking.length > 0')
        .video(v-for='(rank, index) of pleasing_ranking')
          h4 {{ index + 1 }}.
          video(autoplay loop :title='sample_names[rank.name]')
            source(:src='webmUrl(rank.name)' type='video/webm')
            source(:src='mp4Url(rank.name)' type='video/mp4')
            | Can't play video; your browser doesn't support HTML5 video in WebM with VP8/VP9 or MP4 with H.264.
        h4 Relative rank distance
        .plot
          .line
          .point(v-for='(point, index) of pleasing_points' :style='{ left: point * 100 + "%" }')
            .circle
            .label {{ index + 1 }}
      .loading(v-else) Loading...
    section.questionaire
      h2 Post-questionaire
      section
        likert-scale(
          statement='I agree with the ranking of the plants shown above'
          scale='agreement'
          v-model='agree'
        )
      section
        h4 What would you say differentiates good plants vs bad plants in the above ranking?
        textarea(v-model='differentiates')
      section
        h4 Other comments?
        textarea(v-model='comments')
    section.submit
      p
        button(@click='submit()') Submit
</template>

<script>
import Vue from 'vue'
import { last } from 'lodash'
import { API_BASE } from '../config'
import { get, put } from 'axios'
import LikertScale from './LikertScale'

export default {
  components: {
    LikertScale,
  },
  props: {
    token: {
      type: String,
      required: true,
    },
  },
  data () {
    return {
      pleasing_ranking: [],
      sample_names: {},
      agree: undefined,
      differentiates: undefined,
      comments: null,
    }
  },
  computed: {
    realistic_points () {
      return this.calculatePoints(this.realistic_ranking)
    },
    pleasing_points () {
      return this.calculatePoints(this.pleasing_ranking)
    },
    technical_points () {
      return this.calculatePoints(this.technical_ranking)
    },
  },
  methods: {
    mp4Url (id) {
      return `${API_BASE}/video/${id}/mp4`
    },
    webmUrl (id) {
      return `${API_BASE}/video/${id}/webm`
    },
    calculatePoints (ranking) {
      let points = [0]
      let previous = 0
      for (let i = 0; i < ranking.length - 1; ++i) {
        let next = previous + ranking[i].weight - ranking[i + 1].weight
        points.push(next)
        previous = next
      }

      let final = last(points)
      let scaler = 1 / final
      points = points.map(x => x * scaler)

      return points
    },
    fetchNames () {
      for (let rank of this.pleasing_ranking) {
        get(`${API_BASE}/sample/${rank.name}`)
          .then(response => {
            Vue.set(this.sample_names, rank.name, response.data.name)
          })
          .catch(error => console.error('Failed retrieving task', error))
      }
    },
    submit () {
      put(`${API_BASE}/user/${this.token}/post`, {
        ranking_agree: this.agree,
        differentiates: this.differentiates,
        comments: this.comments,
      })
        .then(response => {
          this.$router.push({
            name: 'thanks',
            params: {
              token: this.token,
            },
          })
        })
        .catch(error => console.error('Failed registering user', error))
    },
  },
  created () {
    get(`${API_BASE}/ranking/${this.token}/pleasing`)
      .then(response => {
        this.pleasing_ranking = response.data
        this.fetchNames()
      })
      .catch(error => console.error('Failed retrieving pleasing ranking', error))
  },
}
</script>

<style scoped lang="sass">
.result
  width: 100%
  height: 100%
  padding-top: 20px
  background: #282828
  color: white

section.metric
  max-width: 100%
  margin-bottom: 30px

  h2
    margin-bottom: 5px

  p.details
    text-align: center
    font-style: italic
    opacity: 0.8

.video
  display: inline-block
  margin: 0 10px

  >h4
    margin-bottom: 8px

  >video
    width: 200px
    height: 200px

.plot
  margin: 0 100px
  height: 33px
  position: relative

  .line
    position: absolute
    top: 4px
    height: 2px
    background: white
    width: 100%

  .point
    position: absolute

    >.circle
      width: 10px
      height: 10px
      background: white
      border-radius: 50%
    >.label
      margin-top: 5px
</style>
