<template lang="pug">
  .pairwise
    .video-pair
      video(autoplay loop)
        source(src="static/video/2017-10-16T18:24:47.536062191+02:00.mp4")
        | Can't play video; your browser doesn't support HTML5 video in WebM with VP8/VP9 or MP4 with H.264.
      video(autoplay loop)
        source(src="static/video/2017-10-16T18:00:01.404676397+02:00.mp4")
        | Can't play video; your browser doesn't support HTML5 video in WebM with VP8/VP9 or MP4 with H.264.
</template>

<script>
export default {
  data () {
    return {
    }
  }
}
</script>

<style scoped lang="sass">
.video-pair
  >video
    margin: 10px
</style>
