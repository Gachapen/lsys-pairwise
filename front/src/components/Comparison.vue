<template lang="pug">
  .comparison
    .video-pair
      video(autoplay loop)
        source(v-if='srcA' :src='srcA' type='video/mp4')
        | Can't play video; your browser doesn't support HTML5 video in WebM with VP8/VP9 or MP4 with H.264.
      video(autoplay loop)
        source(v-if='srcB' :src='srcB' type='video/mp4')
        | Can't play video; your browser doesn't support HTML5 video in WebM with VP8/VP9 or MP4 with H.264.
    comparison-slider.slider.realistic(equal='realistic' more='more realistic' :weight.sync='realistic')
    comparison-slider.slider.pleasing(equal='pleasing' more='more pleasing' :weight.sync='pleasing')
</template>

<script>
import ComparisonSlider from './ComparisonSlider'
import { API_BASE } from '../config'

export default {
  components: {
    ComparisonSlider,
  },
  props: {
    pair: {
      type: Object,
      required: true,
    },
  },
  data () {
    return {
      realistic: undefined,
      pleasing: undefined,
    }
  },
  computed: {
    srcA () {
      if (this.pair.a) {
        return `${API_BASE}/video/${this.pair.a}.mp4`
      } else {
        return null
      }
    },
    srcB () {
      if (this.pair.b) {
        return `${API_BASE}/video/${this.pair.b}.mp4`
      } else {
        return null
      }
    },
  },
  watch: {
    realistic (weight) {
      this.$emit('update:realistic', weight)
    },
    pleasing (weight) {
      this.$emit('update:pleasing', weight)
    },
  },
}
</script>

<style scoped lang="sass">
.comparison
  background-color: rgb(127, 127, 127)
  width: 1250px
  margin: 0 auto

  >.video-pair
    >video
      margin: 10px
      width: auto
      height: auto

  >.slider
    margin-top: 10px
    padding: 15px
    background: rgb(137, 137, 137)

  >button.next
    margin-top: 10px
    padding: 10px 20px
    border: 0
    font-size: 12pt
    background: rgb(200, 200, 200)
</style>
