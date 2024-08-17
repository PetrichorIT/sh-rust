<template>
    <v-card
        class="playerContainer"
        width="150px"
        height="220px"
        :style="`border: 10px solid rgb(243, 226, 198); border-radius: 10px; overflow: visible`"
        :loading="player.hasTask ? 'white' : false"
    >
        <v-card-text style="position: relative; z-index: 1">
            <br />

            <v-img
                height="100px"
                :src="'/img/portraits/' + player.user.image"
                style="z-index: 3"
            ></v-img>
            <h3 class="text-center" :style="`color: ${player.user.color};`">
                {{ player.user.name }}
            </h3>

            <v-row align="center" no-gutters>
                <v-col v-if="player.id === state.board.current_president">
                    <v-img :src="`img/position-president.png`" />
                </v-col>

                <v-col v-else-if="player.id === state.state.value?.chancellor">
                    <v-img :src="`img/position-chancellor.png`" />
                </v-col>

                <v-col v-else-if="player.id === state.board.previous_president">
                    <v-img :src="`img/position-chancellor-off.png`" />
                </v-col>

                <v-col
                    v-else-if="player.id === state.board.previous_chancellor"
                >
                    <v-img :src="`img/position-chancellor-off.png`" />
                </v-col>
            </v-row>

            <v-row
                align="start"
                no-gutters
                style="
                    position: absolute;
                    top: 5px;
                    left: 0px;
                    z-index: 2;
                    height: 50px;
                    width: 125px;
                "
            >
                <v-col v-if="player.role !== null">
                    <v-img height="50" :src="`img/role-${player.role}.png`" />
                </v-col>

                <v-col v-if="player.faction !== null">
                    <v-img
                        height="50"
                        :src="`img/faction-${player.faction}.png`"
                    />
                </v-col>

                <v-col v-if="state.board.voting_result !== null">
                    <v-img
                        height="50"
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
