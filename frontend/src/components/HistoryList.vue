<template>
    <v-list max-height="80vh">
        <v-list-item
            class="ma-0 pa-0 text-center"
            v-for="(item, index) in state.board.history"
            v-bind:key="index"
        >
            <v-divider class="mb-2 mt-2" />
            <template v-if="item.ChooseChancellor">
                <v-avatar
                    :image="playerImage(item.ChooseChancellor.president)"
                ></v-avatar>
                <v-icon>mdi-arrow-right</v-icon>
                <v-avatar
                    :image="playerImage(item.ChooseChancellor.chancellor)"
                ></v-avatar
                ><br />
                <br />
                <span>
                    <b>{{ item.ChooseChancellor.president }}</b>
                    selected
                    <b>{{ item.ChooseChancellor.chancellor }}</b> as his
                    chancellor canidate.
                </span>
            </template>
            <template v-if="item.Vote">
                <span>
                    <b>{{ item.Vote.success ? "Successful" : "Failed" }}</b>
                    Vote for president <b>{{ item.Vote.president }}</b> with
                    chancellor
                    <b>{{ item.Vote.chancellor }}</b>
                </span>
                <br />
                <br />
                <v-row align="center" justify="center" no-gutters>
                    <v-col
                        v-for="(vote, key) in item.Vote.votes"
                        v-bind:key="key"
                    >
                        <v-img
                            height="50"
                            :src="vote ? '/img/yes.png' : '/img/no.png'"
                        />
                        <v-avatar
                            class="mt-2"
                            :image="playerImage(key as string)"
                        ></v-avatar>
                    </v-col>
                </v-row>
            </template>
            <template v-if="item.PlayedLaw">
                <v-img
                    height="50"
                    :src="`/img/law-${item.PlayedLaw.law}.png`"
                ></v-img>
                <div class="mt-2">
                    <b>{{ item.PlayedLaw.president }}</b>
                    and
                    <b>{{ item.PlayedLaw.chancellor }}</b> played a
                    <b>{{ item.PlayedLaw.law }}</b> law.
                </div>
            </template>
            <template v-if="item.Veto">
                <v-avatar :image="playerImage(item.Veto.president)"></v-avatar>
                <v-avatar :image="playerImage(item.Veto.chancellor)"></v-avatar>

                <span>
                    <b>{{ item.Veto.president }}</b>
                    and
                    <b>{{ item.Veto.chancellor }}</b> confirmed a veto of the
                    current <b>legislative session</b>.
                </span>
            </template>
        </v-list-item>
    </v-list>
</template>
<script lang="ts">
import { defineComponent } from "vue";
import { GameStateView, PlayerView } from "@/model/types";

export default defineComponent({
    name: "HistoryList",
    props: {
        state: {
            type: Object as () => GameStateView,
            required: true,
        },
    },
    methods: {
        player(id: string): PlayerView | undefined {
            return this.state.board.players.find((p) => p.id == id);
        },

        playerImage(id: string): string {
            return (
                "/img/portraits/" +
                this.state.board.players.find((p) => p.id == id)!.user.image
            );
        },
    },
});
</script>
