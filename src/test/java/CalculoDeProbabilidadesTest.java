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
        assert( 1 ==  res);
    }

    @Test
    void CombinacionesPosibles2(){
        int res = calc.C(0,2);
        assert( 1 ==  res);
    }

    @Test
    void ProbPareja(){

    }

    @Test
    void DecisionPredeterminada(){

        assertEquals( 0 , calc.getDecision().getCodigoDecision() );
    }

    @Test
    void DecisionValida(){

        assert( calc.getDecision().getCodigoDecision() <= 5 && calc.getDecision().getCodigoDecision() >= 0);
    }
}