<template lang="pug">
  .pairwise
    p.cancel
      router-link(:to='`/intro/${token}`') Cancel
    .comparison(v-if='pairs.length > 0')
      .video-pair
        .video(v-html='videoA' ref='videoA')
        .video(v-html='videoB')
      //- comparison-slider.slider.realistic(equal='realistic' more='more realistic' :weight.sync='realistic')
      comparison-slider.slider.pleasing(equal='(dis)pleasing' more='more pleasing' :weight.sync='pleasing' ref='pleasingSlider')
    .loading(v-else)
      p Loading...
    button.next(v-if='!isLast' @click='next' :disabled='!canContinue') Next
    button.finish(v-else @click='finish' :disabled='!canContinue') Finish
</template>

<script>
import axios, { get, post } from 'axios'
import screenfull from 'screenfull'

import ComparisonSlider from './ComparisonSlider'
import { API_BASE } from '../config'

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
        fullscreen: screenfull.isFullscreen,
        video_size: this.$refs.videoA.children[0].offsetHeight,
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
      sources += `Can't play video; your browser doesn't support HTML5 video in WebM with VP9 or MP4 with H.264.`

      let poster = require('../assets/loading_frame.png')

      return `<video autoplay muted loop playsinline poster="${poster}">${sources}</video>`
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
  background-color: rgb(127, 127, 127)
  color: #2c3e50

  >button
    margin-top: 10px
    padding: 10px 20px
    border: 0
    font-size: 12pt
    background: rgb(200, 200, 200)
    color: #666666

    &:disabled
      background: rgb(170, 170, 170)
      color: #868686

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
</style>

<style lang="sass">
.comparison > .video-pair > .video > video
  display: block
  width: auto
  height: auto
  margin: 10px

  @media (max-height: 830px), (max-width: 1250px)
    width: 500px
    height: 500px

  @media (max-height: 730px), (max-width: 1050px)
    width: 400px
    height: 400px

  @media (max-height: 630px), (max-width: 850px)
    width: 300px
    height: 300px

  @media (max-height: 530px), (max-width: 650px)
    width: 200px
    height: 200px

  @media (max-height: 430px), (max-width: 450px)
    width: 150px
    height: 150px

  @media (max-height: 370px), (max-width: 350px)
    width: 125px
    height: 125px

  @media (max-height: 345px), (max-width: 300px)
    width: 100px
    height: 100px
</style>
