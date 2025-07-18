package com.app.service.domain.ports.input;

import com.app.service.domain.model.Carta;

import java.util.List;

public interface CardsUseCases {

    /**
     *
     * @param pocketCards The cards that only the player sees
     * @param communityCards The cards that everyone sees
     */
    void getStats(List<Carta> pocketCards, List<Carta> communityCards);

}
