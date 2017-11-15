<template lang="pug">
  .consent
    section.consent
      h2 Evaluating Generated Virtual Plants
      p.
        You are being asked to participate in a study investigating ways to improve the process of creating computer generated plants.
        These plants could be used in virtual environments, such as video games, movies and simulations.
      p.
        The computer program that generates the plants needs to be able to distinguish good plants from bad plants.
        Your contribution in this study will be on evaluating how well this program does exactly that.
      p.
        If you agree to participate, you will complete three activities: a pre-questionnaire, evaluating pairs of plants, and a post-questionnaire.
        The whole process is expected to take around 10 minutes.
      p.
        Data that you input in fields will be collected.
        Additionally, some data will be automatically collected, limited to your browser name and version, plant video dimensions, submission times and if fullscreen is enabled or not.
        If you received a URL shared by another participant, who this participant was will also be collected.
        All data collected is anonymous and confidential.
      p.
        Your participation is voluntary and you may withdraw at any time during the study without prejudice.
        You may contact the researcher (Magnus Bjerke Vik #[a(href='mailto:magnusbv@stud.ntnu.no') &lt;magnusbv@stud.ntnu.no&gt;]) at any time.
      p
        input(type='checkbox' id='agree' v-model='agree' style='margin-right: 10px')
        label(for='agree')
          b I have read the above, understand the nature of the study, and agree to participate.
      p.
        It is recommended to use a device with a large screen, such as a desktop computer, laptop or tablet to observe the plants in more detail.
      p
        button(:disabled='!agree' @click='$router.push({ path: "register", query: $router.currentRoute.query })') Participate
</template>

<script>
export default {
  data () {
    return {
      agree: false,
    }
  },
}
</script>

<style scoped lang="sass">
</style>
