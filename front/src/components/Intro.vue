<template lang="pug">
  .intro
    section.info
      h2 Description
      p.
        During this experiment you will be asked to rate how much more aesthetically pleasing a plant is
        compared to another. If they are equally pleasing, select '='. If one plant is more
        pleasing than the other, select 1, 2 or 3 on the same side as that plant to indicate how
        much more pleasing it is. At the end you will be presented with the results and some questions
        to answer. It is recommended to
        #[span.link(v-if='canFullscreen' @click='goFullscreen') go fullscreen]
        #[span(v-else) go fullscreen]
        so that you get a bigger view of the plants and less distraction.
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
  created () {
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
