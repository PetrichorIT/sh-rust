<template>
    <v-card
        class="playerContainer"
        width="150px"
        height="200px"
        :style="`background-color: ${player.user.color}; overflow: visible`"
        :loading="player.hasTask ? 'white' : false"
    >
        <v-card-text>
            <v-img
                height="100px"
                :src="'/img/portraits/' + player.user.image"
            ></v-img>
            <h1 class="text-center" style="color: white">
                {{ player.user.name }}
            </h1>

            <v-row align="center" no-gutters>
                <v-col v-if="player.role !== null">
                    <v-img height="75" :src="`img/role-${player.role}.png`" />
                </v-col>

                <v-col v-if="player.faction !== null">
                    <v-img
                        height="75"
                        :src="`img/faction-${player.faction}.png`"
                    />
                </v-col>

                <v-col v-if="player.id === state.board.current_president">
                    <v-avatar :image="`img/star-President.png`" />
                </v-col>
                <v-col v-if="player.id === state.state.value?.chancellor">
                    <v-avatar :image="`img/star-Chancellor.png`" />
                </v-col>

                <v-col v-if="player.id === state.board.previous_president">
                    <v-avatar :image="`img/star-crossed-President.png`" />
                </v-col>

                <v-col v-if="player.id === state.board.previous_chancellor">
                    <v-avatar :image="`img/star-crossed-Chancellor.png`" />
                </v-col>

                <v-col v-if="state.board.voting_result !== null">
                    <v-img
                        height="75"
                        :src="`img/${state.board.voting_result[player.id] ? 'yes' : 'no'}.png`"
                    />
                </v-col>
            </v-row>
        </v-card-text>
    </v-card>
</template>

<script lang="ts">
import { defineComponent } from "vue";
import { GameStateView, PlayerView } from "src/model/types";

export default defineComponent({
    name: "PlayerCard",
    props: {
        player: {
            type: Object as () => PlayerView,
            required: true,
        },
        state: {
            type: Object as () => GameStateView,
            required: true,
        },
    },
});
</script>
<style scoped>
.playerContainer {
    border-radius: 5px;
    text-align: center;
    z-index: 99;
}
</style>
