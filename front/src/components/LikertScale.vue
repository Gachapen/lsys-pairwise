<template lang="pug">
  .likert-scale
    h4.statement {{ statement }}
    p.details(v-if='details') {{ details }}
    .slider
      template(v-for='(point, index) of points')
        label
          input(
            type='radio'
            :name='statement'
            :value='value',
            @change='updateValue(index)'
          )
          | {{ point }}
</template>

<script>
export default {
  props: {
    statement: {
      type: String,
      required: true,
    },
    details: {
      type: String,
      required: false,
    },
    scale: {
      type: String,
      required: false,
      default: 'agreement',
    },
    value: {
      type: Number,
      required: false,
    },
  },
  data () {
    return {
    }
  },
  computed: {
    points () {
      if (this.scale === 'agreement') {
        return [
          'strongly disagree',
          'disagree',
          'neutral',
          'agree',
          'strongly agree',
        ]
      }

      if (this.scale === 'frequency') {
        return [
          'never',
          'rarely',
          'occasionally',
          'frequently',
          'very frequently',
        ]
      }

      if (this.scale === 'like') {
        return [
          'hate',
          'dislike',
          'neutral',
          'like',
          'love',
        ]
      }

      throw new Error(`'${this.scale}' scale not supported`)
    },
  },
  methods: {
    updateValue (value) {
      // Make it in range [-2, 2].
      value -= 2
      this.$emit('input', value)
    },
  },
}
</script>

<style scoped lang="sass">
.likert-scale
  h4.statement
    margin-bottom: 5px
    text-align: left

  p.details
    font-style: italic
    margin-top: 0
    margin-bottom: 0
    text-align: left

  >.slider
    margin-top: 10px
    width: 100%
    display: flex
    justify-content: space-around

    >label
      vertical-align: top
      text-align: center
      width: 100%

      >input
        display: block
        margin: 0 auto
        margin-bottom: 5px
</style>
