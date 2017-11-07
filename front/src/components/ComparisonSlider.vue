<template lang="pug">
  .comparison-slider
    .headings
      .left
        h4 Left is {{ more }}
      .equal
        h4 Equally {{ equal }}
      .right
        h4 Right is {{ more }}
    .slider
      template(v-for='point in points')
        label
          input(type='radio' :name='equal' :value='point.weight' v-model='weight')
          | {{ point.label }}
</template>

<script>
export default {
  props: {
    more: {
      type: String,
      required: true,
    },
    equal: {
      type: String,
      required: true,
    },
  },
  data () {
    return {
      weight: undefined,
    }
  },
  computed: {
    points () {
      return [
        {
          weight: 1 / (1 + (8 / 3 * 3)),
          label: '3',
        },
        {
          weight: 1 / (1 + (8 / 3 * 2)),
          label: '2',
        },
        {
          weight: 1 / (1 + (8 / 3 * 1)),
          label: '1',
        },
        {
          weight: 1,
          label: '=',
        },
        {
          weight: 1 + (8 / 3 * 1),
          label: '1',
        },
        {
          weight: 1 + (8 / 3 * 2),
          label: '2',
        },
        {
          weight: 1 + (8 / 3 * 3),
          label: '3',
        },
      ]
    },
  },
  watch: {
    weight (val) {
      this.$emit('update:weight', val)
    },
  },
  methods: {
    unselect () {
      this.weight = undefined
    },
  },
}
</script>

<style scoped lang="sass">
.comparison-slider
  padding: 15px

  >.headings
    width: 100%
    display: flex
    justify-content: space-evenly

    h4
      margin-top: 10px

  >.slider
    width: 100%
    display: flex
    justify-content: space-around

    >label
      vertical-align: top
      width: 100%

      >input
        display: block
        margin: 0 auto
        margin-bottom: 5px
</style>
