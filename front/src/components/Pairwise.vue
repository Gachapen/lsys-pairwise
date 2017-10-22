<template lang="pug">
  .pairwise
    .comparison
      .video-pair
        video(autoplay loop)
          source(src="static/video/2017-10-16T18:24:47.536062191+02:00.mp4")
          | Can't play video; your browser doesn't support HTML5 video in WebM with VP8/VP9 or MP4 with H.264.
        video(autoplay loop)
          source(src="static/video/2017-10-16T18:00:01.404676397+02:00.mp4")
          | Can't play video; your browser doesn't support HTML5 video in WebM with VP8/VP9 or MP4 with H.264.
      comparison-slider.slider.realistic(equal='realistic' more='more realistic')
      comparison-slider.slider.pleasing(equal='pleasing' more='more pleasing')
      button.next Next
</template>

<script>
import ComparisonSlider from './ComparisonSlider'
export default {
  components: {
    ComparisonSlider,
  },
  data () {
    return {
    }
  },
}
</script>

<style scoped lang="sass">
.pairwise
  width: 100%
  height: 100%
  padding-top: 20px
  background-color: rgb(127, 127, 127)

  >.comparison
    width: 1250px
    margin: 0 auto

    >.video-pair
      >video
        margin: 10px

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
