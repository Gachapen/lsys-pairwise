<template lang="pug">
  .result
    .metric.realistic
      h2 Most realistic
      .video(v-for='(rank, index) of realistic_ranking')
        h4 {{ index + 1 }}.
        video(autoplay loop)
          source(:src='mp4Url(rank.name)' type='video/mp4')
          | Can't play video; your browser doesn't support HTML5 video in WebM with VP8/VP9 or MP4 with H.264.
      .plot
        .line
        .point(v-for='(point, index) of realistic_points' :style='{ left: point * 100 + "%" }')
          .circle
          .label {{ index + 1 }}
    .metric.pleasing
      h2 Most pleasing
      .video(v-for='(rank, index) of pleasing_ranking')
        h4 {{ index + 1 }}.
        video(autoplay loop)
          source(:src='mp4Url(rank.name)' type='video/mp4')
          | Can't play video; your browser doesn't support HTML5 video in WebM with VP8/VP9 or MP4 with H.264.
      .plot
        .line
        .point(v-for='(point, index) of pleasing_points' :style='{ left: point * 100 + "%" }')
          .circle
          .label {{ index + 1 }}
</template>

<script>
import { last } from 'lodash'
import { API_BASE } from '../config'
import { get } from 'axios'

export default {
  components: {
  },
  props: {
    token: {
      type: String,
      required: true,
    },
  },
  data () {
    return {
      realistic_ranking: [],
      pleasing_ranking: [],
    }
  },
  computed: {
    realistic_points () {
      return this.calculatePoints(this.realistic_ranking)
    },
    pleasing_points () {
      return this.calculatePoints(this.pleasing_ranking)
    },
  },
  methods: {
    mp4Url (name) {
      return `${API_BASE}/video/${name}.mp4`
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
  },
  created () {
    get(`${API_BASE}/task/${this.token}/ranking/realistic`)
      .then(response => {
        this.realistic_ranking = response.data
      })
      .catch(error => console.error('Failed retrieving task', error))

    get(`${API_BASE}/task/${this.token}/ranking/pleasing`)
      .then(response => {
        this.pleasing_ranking = response.data
      })
      .catch(error => console.error('Failed retrieving task', error))
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

.metric
  margin-bottom: 30px

  h2
    margin-bottom: 5px

.video
  display: inline-block
  margin: 0 10px

  >h4
    margin-bottom: 8px

  >video
    width: 250px
    height: 250px

.plot
  width: 1500px
  height: 33px
  margin: 0 auto
  margin-top: 20px
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