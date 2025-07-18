package com.app.service;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.ArrayList;
import java.util.List;

import org.junit.jupiter.api.Test;

import com.app.service.applications.service.CalculoDeProbabilidades;
import com.app.service.domain.model.Carta;
import com.app.service.applications.service.Decision;

public class DecisionTest {
	
	private CalculoDeProbabilidades calc;
	private List<Carta> cartasEscaleraReal;;

	private final int NUM_CONTRINCANTES = 3;
	private final int CIEGA_PEQUENYA = 10;
	private final int APUESTA_ACUMULADA = 200;
	
	public DecisionTest() {
		calc = new CalculoDeProbabilidades();
        cartasEscaleraReal = new ArrayList<>();
        cartasEscaleraReal.add( new Carta('T', 1) );
        cartasEscaleraReal.add( new Carta('T', 13) );
        cartasEscaleraReal.add( new Carta('T', 12) );
        cartasEscaleraReal.add( new Carta('T', 11) );
        cartasEscaleraReal.add( new Carta('T', 10) );
        
        calc.reiniciarDatos(cartasEscaleraReal, NUM_CONTRINCANTES , CIEGA_PEQUENYA, APUESTA_ACUMULADA);
	}
	
	
    @Test
    void DecisionValida(){
		Decision decision = calc.getDecision();
		int decisionNumber = decision.getCodigoDecision();
		
		assertTrue( decisionNumber >= 0 && decisionNumber <= 5);
    }
	
    @Test
    void DecisionCorrecta() {	/* Teniendo la mejor mano del juego no puede salirse de la mano*/
    	Decision decision = calc.getDecision();

    	assertFalse( decision.decideEsto( Decision.FOLD ));
    }
}
