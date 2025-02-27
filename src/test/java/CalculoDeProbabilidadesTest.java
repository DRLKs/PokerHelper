package test.java;
import main.clases.CalculoDeProbabilidades;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;


public class CalculoDeProbabilidadesTest{

    private final int RESULTADO_ERROR_COMBINACIONES = -1;

    CalculoDeProbabilidades calc;

    @BeforeEach
    void setup() {
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

    @Test
    void DecisionPredeterminada(){
    }

    @Test
    void DecisionValida(){
    }
}