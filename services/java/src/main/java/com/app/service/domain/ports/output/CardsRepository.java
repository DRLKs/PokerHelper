package com.app.service.domain.ports.output;

import com.app.service.domain.model.Carta;

import java.util.List;

public interface CardsRepository {

    void getStats(List<Carta> pocketCards, List<Carta> communityCards);

}
