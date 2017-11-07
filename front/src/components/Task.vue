<template lang="pug">
  .pairwise
    .comparison(v-if='pairs.length > 0')
      .video-pair
        .video(v-html='videoA')
        .video(v-html='videoB')
      //- comparison-slider.slider.realistic(equal='realistic' more='more realistic' :weight.sync='realistic')
      comparison-slider.slider.pleasing(equal='(dis)pleasing' more='more pleasing' :weight.sync='pleasing' ref='pleasingSlider')
    .loading(v-else)
      p Loading...
    button.next(v-if='!isLast' @click='next' :disabled='!canContinue') Next
    button.finish(v-else @click='finish' :disabled='!canContinue') Finish
</template>

<script>
import ComparisonSlider from './ComparisonSlider'
import { API_BASE } from '../config'
import axios, { get, post } from 'axios'

export default {
  components: {
    ComparisonSlider,
  },
  props: {
    token: {
      type: String,
      required: true,
    },
  },
  data () {
    return {
      pairs: [],
      pairIndex: 0,
      // realistic: undefined,
      pleasing: undefined,
      videoTypes: ['webm', 'mp4'],
    }
  },
  computed: {
    currentPair () {
      if (this.pairIndex < this.pairs.length) {
        return this.pairs[this.pairIndex]
      } else {
        return {
          a: undefined,
          b: undefined,
        }
      }
    },
    pairId () {
      return `${this.currentPair.a} ${this.currentPair.b}`
    },
    canContinue () {
      // return this.realistic && this.pleasing
      return this.pleasing
    },
    isLast () {
      return this.pairIndex === this.pairs.length - 1
    },
    videoA () {
      if (this.currentPair.a) {
        return this.constructVideoElement(this.currentPair.a)
      } else {
        return ''
      }
    },
    videoB () {
      if (this.currentPair.b) {
        return this.constructVideoElement(this.currentPair.b)
      } else {
        return ''
      }
    },
  },
  methods: {
    postWeight (metric, weight) {
      return post(`${API_BASE}/weight`, {
        token: this.token,
        metric,
        a: this.currentPair.a,
        b: this.currentPair.b,
        weight,
      })
    },
    next () {
      axios.all([
        // this.postWeight('realistic', this.realistic),
        this.postWeight('pleasing', this.pleasing),
      ])
        .then(() => {
          this.pairIndex += 1
          // this.realistic = undefined
          this.pleasing = undefined
          this.$refs.pleasingSlider.unselect()
        })
        .catch(error => console.error('Failed posting weights', error))
    },
    finish () {
      axios.all([
        // this.postWeight('realistic', this.realistic),
        this.postWeight('pleasing', this.pleasing),
      ])
        .then(() => {
          this.$router.push({ name: 'result', params: { token: this.token } })
        })
        .catch(error => console.error('Failed posting weights', error))
    },
    constructVideoElement (sampleId) {
      const baseSrc = `${API_BASE}/video/${sampleId}`

      let sources = ''
      for (let type of this.videoTypes) {
        sources += `<source src='${baseSrc}/${type}' type='video/${type}'>`
      }
      sources += `Can't play video; your browser doesn't support HTML5 video in WebM with VP8/VP9 or MP4 with H.264.`

      return `<video autoplay="true" loop="true">${sources}</video>`
    },
  },
  created () {
    get(`${API_BASE}/task/${this.token}`)
      .then(response => {
        this.pairs = response.data
        this.pairIndex = 0

        if (this.pairs.length === 0) {
          this.$router.push({ name: 'result', params: { token: this.token } })
        }
      })
      .catch(error => console.error('Failed retrieving task', error))
  },
}
</script>

<style scoped lang="sass">
.pairwise
  width: 100%
  height: 100%
  padding-top: 20px
  background-color: rgb(127, 127, 127)

  >button
    margin-top: 10px
    padding: 10px 20px
    border: 0
    font-size: 12pt
    background: rgb(200, 200, 200)

.comparison
  background-color: rgb(127, 127, 127)
  max-width: 1250px
  margin: 0 auto

  >.video-pair
    >.video
      display: inline-block
      width: auto
      height: auto

  >.slider
    background: rgb(137, 137, 137)

  >button.next
    margin-top: 10px
    padding: 10px 20px
    border: 0
    font-size: 12pt
    background: rgb(200, 200, 200)
</style>

<style lang="sass">
.comparison > .video-pair > .video > video
  display: block
  width: auto
  height: auto
  margin: 10px

  @media (max-width: 1400px)
    width: 500px
    height: 500px

  @media (max-width: 1100px)
    width: 300px
    height: 300px
</style>
