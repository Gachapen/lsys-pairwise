<template lang="pug">
  .thanks
    section
      h1 Thank you for participating!
      p
        router-link(:to='`/result/${token}?no-questions=true`') Save this link to see your ranking again!
</template>

<script>
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
    }
  },
  computed: {
  },
  methods: {
  },
  created () {
  }
}
</script>

<style scoped lang="sass">
p
  text-align: center
</style>
