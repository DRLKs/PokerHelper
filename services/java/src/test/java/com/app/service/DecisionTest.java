package com.app.service;

import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.ArrayList;
import java.util.List;

import com.pokerhelper.application.services.CalculoDeProbabilidades;
import com.pokerhelper.domain.model.Card;
import com.pokerhelper.domain.model.Decision;
import org.junit.jupiter.api.Test;


public class DecisionTest {
	
	private CalculoDeProbabilidades calc;
	private List<Card> cartasEscaleraReal;;

	private final int NUM_CONTRINCANTES = 3;
	private final int CIEGA_PEQUENYA = 10;
	private final int APUESTA_ACUMULADA = 200;
	
	public DecisionTest() {
		calc = new CalculoDeProbabilidades();
        cartasEscaleraReal = new ArrayList<>();
        cartasEscaleraReal.add( new Card('T', 1) );
        cartasEscaleraReal.add( new Card('T', 13) );
        cartasEscaleraReal.add( new Card('T', 12) );
        cartasEscaleraReal.add( new Card('T', 11) );
        cartasEscaleraReal.add( new Card('T', 10) );
        
        calc.reiniciarDatos(cartasEscaleraReal, NUM_CONTRINCANTES , CIEGA_PEQUENYA, APUESTA_ACUMULADA);
	}
	
	
    @Test
    void DecisionValida(){
		Decision decision = calc.getDecision();
        int decisionActionNumber = decision.getAction();

		assertTrue( decisionActionNumber >= 0 && decisionActionNumber <= 5);
    }
	
    @Test
    void DecisionCorrecta() {	/* Teniendo la mejor mano del juego no puede salirse de la mano*/
    	Decision decision = calc.getDecision();

    	assertNotEquals( Decision.FOLD, decision.getAction());
    }
}
