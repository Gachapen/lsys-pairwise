<template lang="pug">
  .intro
    section.info
      h2 Description
      p.
        During this experiment you will be asked to rate how much more aesthetically pleasing a plant is
        compared to another. If they are equally pleasing, select '='. If one plant is more
        pleasing than the other, select 1, 2 or 3 on the same side as that plant to indicate how
        much more pleasing it is. At the end you will be presented with the results and some questions
        to answer.
      p.
        It is recommended to
        #[span.link(v-if='canFullscreen' @click='goFullscreen') #[b go fullscreen]]
        #[span(v-else) go fullscreen]
        so that you get a bigger view of the plants and less distraction.
        You can press the ESC key (or F11) to exit fullscreen.
      p.
        There will always be a 'cancel' link at the top of the page to go back to this page if you
        have any issues or would like to withdraw (you can continue from where you left).
      p.
        If, during the plant evaluation, you do not see two videos side by side of rotating plants, please try a different browser (copy the URL), or withdraw from the study as it may invalidate the results.
        Please contact the researcher (Magnus Bjerke Vik #[a(href='mailto:magnusbv@stud.ntnu.no') &lt;magnusbv@stud.ntnu.no&gt;]) if you notice any issues.
      p#token
        label(for='token-out')
          span Your token is:
        output#token-out {{ token }}
      p
        button(@click='$router.push({ name: "task", params: { token: token } })') Begin
</template>

<script>
import screenfull from 'screenfull'

export default {
  props: {
    token: {
      type: String,
      required: true,
    },
  },
  data () {
    return {
    }
  },
  computed: {
    canFullscreen () {
      return screenfull.enabled
    },
  },
  methods: {
    goFullscreen () {
      if (screenfull.enabled) {
        screenfull.request()
      }
    },
  },
  mounted () {
    if (screenfull.enabled && screenfull.isFullscreen) {
      screenfull.exit()
    }
  },
}
</script>

<style scoped lang="sass">
#token
  opacity: 0.4

  >label >span
    margin-right: 10px

.link
  text-decoration: underline
  cursor: pointer
</style>
