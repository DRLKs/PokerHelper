package com.app.service;
import com.pokerhelper.application.services.CalculoDeProbabilidades;
import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.assertEquals;



public class CalculoDeProbabilidadesTest{

    private final int RESULTADO_ERROR_COMBINACIONES = -1;

    private CalculoDeProbabilidades calc;

    public CalculoDeProbabilidadesTest() {
    	calc = new CalculoDeProbabilidades();
    }

    @Test
    void CombinacionesImposibles1(){
        int res = calc.C(-1,-2);
        assertEquals( RESULTADO_ERROR_COMBINACIONES, res);
    }

    @Test
    void CombinacionesImposibles2(){
        int res = calc.C(-1,2);
        assertEquals( RESULTADO_ERROR_COMBINACIONES, res);
    }

    @Test
    void CombinacionesImposibles3(){
        int res = calc.C(1,-2);
        assertEquals( RESULTADO_ERROR_COMBINACIONES, res);
    }

    @Test
    void CombinacionesPosibles1(){
        int res = calc.C(2,2);
        assertEquals( 1, res);
    }

    @Test
    void CombinacionesPosibles2(){
    	for( int n = 1 ; n < 10 ; ++n ) {
    		int res = calc.C(n,n-1);
            assertEquals( n,  res);
    	}
    }

    @Test
    void ProbPareja(){

    }
}