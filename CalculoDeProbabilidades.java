
public class CalculoDeProbabilidades {
	
	public int C( int n1 ,int n2) { // COMBINACIONES
		
		if( n1 < n2 ) { // ERROR
			return 0;
		}
		
		int n3 = n1 - n2;
		int totalNumerador = 1;
		for( int i = n3 + 1 ; i <= n1 ; ++i ) {
			totalNumerador *= i;
		}
		int totalDenominador = 1;
		for( int i = 2 ; i <= n2 ; ++i ) {
			totalDenominador *= i;
		}
		return totalNumerador / totalDenominador;
	}
	
		
}
