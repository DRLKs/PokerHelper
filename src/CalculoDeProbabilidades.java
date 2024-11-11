import java.util.HashSet;
import java.util.List;

public class CalculoDeProbabilidades {
	/*
	 * Todas las probabilidades son sobre 1
	 */
	private double probEscalera;
	private double probColor;
	private double probFullHouse;
	private double probPoker;
	private double probEscaleraColor;
	private double probEscaleraReal;
	
	/*
	 * Recalcula toda la información
	 * 
	 * Se llamará cuando se actualicen los datos
	 */
	public void reiniciarDatos( List<Carta> cartas ) {
		probEscalera = completarEscalera(cartas);
		probColor = completarColor(cartas);
		probFullHouse = completarFullHouse(cartas);
		probPoker = completarPoker(cartas);
		probEscaleraColor = completarEscaleraColor(cartas);
		probEscaleraReal = completarEscaleraReal(cartas);
	}
	
	public double getProbEscalera() {
		return probEscalera;
	}

	public double getProbColor() {
		return probColor;
	}

	public double getProbFullHouse() {
		return probFullHouse;
	}

	public double getProbPoker() {
		return probPoker;
	}

	public double getProbEscaleraColor() {
		return probEscaleraColor;
	}

	public double getProbEscaleraReal() {
		return probEscaleraReal;
	}

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
	
	/* 
	 * Estas Funciones calcularán las posibilidades de obtener color
	 * numCartasColor: 	son las cartas de **** que tienes en tu mano
	 * ronda:			es la ronda, parte de la mano en la que nos encontramos
	 * 		0 = Solo tenemos nuestras cartas
	 * 		1 = Tenemos nuestras cartas y 3 más en la mesa (FLOP)
	 * 		2 = Tenemos nuestras cartas y 4 más	(TURN)
	 * 		3 = Tenemos todas las cartas  (RIVER)
	 * 
	 * 	CABE DESTACAR: El número de jugadores no interviene en las probabilidades que haya de obtener tu mano deseada
	 * 
	 * */
	private double completarColor( List<Carta> cartas ){
		
		double prob = 0.0;
		int numCartasColor = 1;
		int numCartas = cartas.size();
		int cartasPorMostrar = 5 - (numCartas - 2);
		
		if( cartas.get(0).mismoPaloQue( cartas.get(1) ) ) {
			++numCartasColor;
			
			for( int i = 2; i < numCartas ; ++i ) {
				if( cartas.get(i).mismoPaloQue(cartas.get(0)) ) {
					++numCartasColor;
				}
			}
			
			prob = probCompletarColor(numCartasColor, cartasPorMostrar );
		}else {

			for( int idxCMano = 0 ; idxCMano < 2 ; ++idxCMano ) {	// Comrpobar que nuestras cartas participen en estos datos
				for( int idx = 0; idx < 2 ; ++idx ) {
					for( int i = 2; i < numCartas ; ++i ) {
						if( cartas.get(i).mismoPaloQue(cartas.get(idx)) ) {
							++numCartasColor;
						}
					}
					prob += probCompletarColor(numCartasColor, cartasPorMostrar);
					numCartasColor = 1;
				}
			}
		}

		return prob;
	}
	
	private double probCompletarColor( int numCartasColor, int cartasPorMostrar ) {
		
		double prob = 0.0;
		int cartasNecesarias = 5 - numCartasColor;
		int cartasEseTipoRestantes = 13 - numCartasColor;
		if( cartasNecesarias <=0 ) {
			prob = 1.0;
		}else if( cartasPorMostrar > 0 ){
			int cartasQueNoHanSalido = 50 - (5 - cartasPorMostrar);
			
			prob = 1.0;
			int combinacionesPosibles = C(cartasPorMostrar,cartasNecesarias);
			while( cartasNecesarias > 0 ) {
				prob *= (double ) cartasEseTipoRestantes / cartasQueNoHanSalido;
				--cartasEseTipoRestantes;
				--cartasQueNoHanSalido;
				--cartasNecesarias;
			}
			prob *= combinacionesPosibles;	// Esto no es del todo correcto, perdemos precisión
			
		}
		return prob ;
	}
	/*
	 * Al encontrarnos con varias cartas, hay diferentes posibles escaleras:
	 * 	
	 * 	Esto hará variar nuestra probabilidad de obtener mano, existen varias opciones:
	 * 		- Solo haya una opción posible. Ej: 2, 3, 6, J, Q, ?, ? -> 4 y 5
	 * 		- Haya 2 opciones posibles.		Ej: 2, 3, 4, 5, Q, ?, ? -> A, 6, 6 
	 * 		- Haya 3 opciones posibles.		Ej:  
	 * 		- Haya 4 opciones posibles.		Ej: 2, 3, 4 , A, J, ?, ?
	 * 		- Haya 
	 * 	
	 */
	private double completarEscalera( List<Carta> cartas ){
		
		double prob = 0.0;
		/* Reconocemos la ronda en la que nos encontramos*/
		int ronda = 0;
		int numCartas = cartas.size();
		if( numCartas == 5 ) {
			ronda = 1;
		}else if( numCartas == 6 ) {
			ronda = 2;
		}else if( numCartas == 7 ){
			ronda = 3;
		}
		
		int numCartasEscalera = 1;
		
		/* Vamos a descubrir las posibles escaleras */
		HashSet<Integer> numerosDeLasCartasMano = new HashSet<>();
		numerosDeLasCartasMano.add( cartas.get(0).getNumero() );
		numerosDeLasCartasMano.add( cartas.get(1).getNumero() );

		HashSet<Integer> numerosDeLasCartas = new HashSet<>();
		for( Carta carta : cartas ) {
			int numCarta = carta.getNumero();
			if( numCarta == 14 ) {	// Para poder encontrar las escaleras bajas
				numerosDeLasCartas.add( 1 );
			}
			numerosDeLasCartas.add( numCarta );
		}
			
			
			
		
		/*
		if( cartas.get(0).puedenHacerEscalera( cartas.get(1) ) ) {
			++numCartasEscalera;
			
			for( int i = 2; i < numCartas ; ++i ) {
				if( cartas.get(i).puedenHacerEscalera(cartas.get(0)) ) {
					++numCartasEscalera;
				}
			}
			
			prob = probCompletarEscalera(numCartasEscalera, ronda, 1 );
		}else {
			prob = 0.0;
			for( int idxCMano = 0 ; idxCMano < 2 ; ++idxCMano ) {
				for( int idx = 0; idx < 2 ; ++idx ) {
					for( int i = 2; i < numCartas ; ++i ) {
						if( cartas.get(i).puedenHacerEscalera(cartas.get(idx)) && cartas.get(idxCMano).puedenHacerEscalera(cartas.get(i))) {
							++numCartasEscalera;
						}
					}
				prob += probCompletarEscalera(numCartasEscalera, ronda, 1);
				numCartasEscalera = 1;
				}
			}
			
		}
		*/
		return prob;
	}
	
	private double probCompletarEscalera( int numCartasEscalera, int ronda, int numPosiblesEscaleras ){
		
		double prob = 0.0;

		int cartasNecesarias = 5 - numCartasEscalera;
		int cartasEseTipoRestantes = 4 * (cartasNecesarias + numPosiblesEscaleras - 1) ;
		int cartasPorEnsenyar;
		if( cartasNecesarias <=0 ) {
			prob = 1.0;
		}else if( ronda < 3 ){
			
			int cartasQueNoHanSalido;
			if( ronda == 0) {
				cartasQueNoHanSalido = 50;
				cartasPorEnsenyar = 5;
			}else if( ronda == 1 && cartasNecesarias <= 2) {
				cartasQueNoHanSalido = 47;
				cartasPorEnsenyar = 2;
			}else if( ronda == 2 && cartasNecesarias == 1 ) {
				cartasQueNoHanSalido = 46;
				cartasPorEnsenyar = 1;
			}else {
				return 0;	
			}
			int combinacionesPosibles = C(cartasPorEnsenyar,cartasNecesarias);
			prob = 1.0;
			while( cartasNecesarias > 0 ) {
				prob *= (double ) cartasEseTipoRestantes / cartasQueNoHanSalido;
				cartasEseTipoRestantes -= 4;
				--cartasQueNoHanSalido;
				--cartasNecesarias;
			}
			
		}
		return prob;
	}

	private double completarFullHouse( List<Carta> cartas ) {	// Falta por completar
		
		double prob = 0.0;
		int ronda = 0;
		int numCartas = cartas.size();
		if( numCartas == 5 ) {
			ronda = 1;
		}else if( numCartas == 6 ) {
			ronda = 2;
		}else if( numCartas == 7 ){
			ronda = 3;
		}
		
		
		return prob;
	}
	/*
	 * Esta función calcula la probabilidad de que salga FULL HOUSE en estos casos:
	 * 
	 * 	numCartasTrio: Cartas que supuestamente harían trio
	 *  numCartasPareja: Cartas que supuestamente harían pareja
	 *  
	 *  Alguna de estas cartas deben pertenecer a las cartas de nuestra mano
	 */
	private double probCompletarFullHouse( int numCartasTrio, int numCartasPareja, int ronda ){	
		
		double prob = 0.0;
		int cartasNecesariasTrio = 3 - numCartasTrio;
		int cartasNecesariasPareja = 2 - numCartasPareja;
		
		int cartasQueQueremosTrio = 4 - numCartasTrio;
		int cartasQueQueremosPareja = 4 - numCartasPareja;

		
		if( cartasNecesariasTrio <= 0 && cartasNecesariasPareja <= 0) {
			prob = 1.0;
		}else if( ronda < 3 ){
	
			int cartasQueNoHanSalido;
			if( ronda == 0) {
				cartasQueNoHanSalido = 50;
				
			}else if( ronda == 1 && (cartasNecesariasTrio + cartasNecesariasPareja) <= 2) {
				cartasQueNoHanSalido = 47;
			
			}else if( ronda == 2 && (cartasNecesariasTrio + cartasNecesariasPareja) == 1 ) {
				cartasQueNoHanSalido = 46;
			}else {
				return 0;	
			}
			
			prob = 1.0;	
			while( cartasNecesariasTrio > 0 ) {	
				prob *= (double ) cartasQueQueremosTrio / cartasQueNoHanSalido;
				cartasQueNoHanSalido -= 4;
				--cartasNecesariasTrio;
				--cartasQueNoHanSalido;
				--cartasQueQueremosTrio;
			}
			
			while( cartasNecesariasPareja > 0 ) {
				prob *= (double ) cartasQueQueremosPareja / cartasQueNoHanSalido;
				cartasQueNoHanSalido -= 4;
				--cartasNecesariasPareja;
				--cartasQueNoHanSalido;
				--cartasQueQueremosPareja;
			}
			
		}
		return prob;
	}
	
	private double completarPoker( List<Carta> cartas ) {	
		
		double prob;
		
		int numCartas = cartas.size();
		int cartasPorMostrar = 5 - (numCartas-2);	
		int cttCartasPoker = 1;
		if( cartas.get(0).mismoNumeroQue( cartas.get(1) ) ) {
			++cttCartasPoker;
			for(  int i = 2 ; i < numCartas ; ++i ) {
				if(cartas.get(0).mismoNumeroQue(cartas.get(i))) {
					++cttCartasPoker;
				}
			}
			prob = probCompletarPoker( cttCartasPoker , cartasPorMostrar);
		}else {
			prob = probCompletarPoker( 1 , cartasPorMostrar) * 2;
		}
		
		return prob;
	}
	
	private double probCompletarPoker( int numCartasPoker, int cartasPorMostrar ) {
		
		double prob = 0.0;

		int cartasNecesarias = 4 - numCartasPoker;
		int cartasEseTipoRestantes = 4 - numCartasPoker;
		
		if( cartasNecesarias <=0 ) {
			prob = 1.0;
		}else if( cartasPorMostrar > 0 ){
	
			int cartasQueNoHanSalido = 50 - (5 - cartasPorMostrar);
			
			int combinacionesPosibles = C(cartasPorMostrar,cartasNecesarias);
			prob = 1.0;
			while( cartasNecesarias > 0 ) {
				prob *= (double ) cartasEseTipoRestantes / cartasQueNoHanSalido;
				--cartasEseTipoRestantes;
				--cartasQueNoHanSalido;
				--cartasNecesarias;
			}
			prob *= combinacionesPosibles;	// No del todo correcto
		}
		return prob;
	}
	
	private double completarEscaleraColor( List<Carta> cartas ){
		
		double prob = 0.0;
		int numCartas = cartas.size();
		int cartasPorMostrar = 5 - (numCartas - 2);
		int arrayInicio;
		int arratFinal;
		
		// 0 = No tenemos ese número || 1 = Tenemos ese número
		int[] numerosEscaleraColor;	
		
		// Nuestras 2 pueden hacer escalera color
		if( cartas.get(0).puedenHacerEscalera( cartas.get(1) ) &&  cartas.get(0).mismoPaloQue( cartas.get(1))) {	
			numerosEscaleraColor = new int[14];
			numerosEscaleraColor[ cartas.get(0).getNumero() - 1 ] = 1;
			numerosEscaleraColor[ cartas.get(1).getNumero() - 1 ] = 1;
			
			arrayInicio = cartas.get(0).getNumero() - 4;
			for( int i = 2; i < numCartas ; ++i ) {
				if( cartas.get(0).puedenHacerEscalera(cartas.get(i)) && cartas.get(0).mismoPaloQue(cartas.get(i)) ) {
					numerosEscaleraColor[ cartas.get(i).getNumero() - 1 ] = 1;
				}
			}
			if( numerosEscaleraColor[13] == 1 ) {	// Está el AS, 14-1=13
				numerosEscaleraColor[0] = 1;
			}
			prob = probCompletarEscaleraColor(numerosEscaleraColor, cartasPorMostrar, arrayInicio);
		}else {	// Nuestras 2 cartas no pueden hacer escalera de color
			for( int idxCMano = 0 ; idxCMano < 2 ; ++idxCMano ) {	// CREO QUE ESTÁ MAL
					numerosEscaleraColor = new int[14];
					for( int idx = 2; idx < numCartas ; ++idx ) {
						if( 
							cartas.get(idxCMano).mismoPaloQue(cartas.get(idx))	&&
							cartas.get(idx).puedenHacerEscalera(cartas.get(idx))
							) {	
					}
					prob += probCompletarEscaleraColor(numerosEscaleraColor, cartasPorMostrar, arrayInicio);
				}
			}
		}
		
		return prob;
	}
	
	private double probCompletarEscaleraColor( int[] numerosEscaleraColor, int cartasPorMostrar, int arrayInicio ){	// FALTAN VER CARTAS EXTREMOS Y ESAS COSAS
		
		double prob = 0.0;
		
		int cartasNecesarias = 5 - numCartasEscalera;
		int cartasEseTipoRestantes = cartasNecesarias + numPosiblesEscaleras - 1;
		
		if( cartasNecesarias <=0 ) {
			prob = 1.0;
		}else if( cartasPorMostrar > 0 ){
	
			int cartasQueNoHanSalido = 50 - (5-cartasPorMostrar);  
			
			prob = 1.0;
			while( cartasNecesarias > 0 ) {
				prob *= (double ) cartasEseTipoRestantes / cartasQueNoHanSalido;
				--cartasEseTipoRestantes;
				--cartasQueNoHanSalido;
				--cartasNecesarias;
			}
			
		}
		return prob;
	}
	
	private double completarEscaleraReal( List<Carta> cartas ){
		
		double prob = 0.0;
		int numCartas = cartas.size();
		int cartasPorMostrar = 5 -(numCartas-2);
		int numCartasEscalera = 1;
		boolean cartaMano1Posible = false;
		boolean cartaMano2Posible = false;
		/*
		 * Para hacerlo má seficiente
		 */
		int inicioBucle = 1;
		int topeBucle = 0;
		
		/* Comprobamos las cartas de nuestra mano sean aptas para la escalera real*/
		if( cartas.get(0).getNumero() >= 10 ) {
			cartaMano1Posible = true;
			inicioBucle = 0;
		}
		if( cartas.get(1).getNumero() >= 10 ) {
			cartaMano2Posible = true;
			topeBucle = 1;
		}
		
		if( cartas.get(0).mismoPaloQue( cartas.get(1)) && cartaMano1Posible && cartaMano2Posible) {
			++numCartasEscalera;
			topeBucle = 0;
		}
		
		for( int idxCartaMano = inicioBucle ; idxCartaMano <= topeBucle ; ++idxCartaMano ) {
			for( int idxCartaMesa = 2; idxCartaMesa < numCartas ; ++idxCartaMesa ) {
				if( 
					cartas.get(idxCartaMesa).getNumero() >= 10	&&
					cartas.get(idxCartaMano).mismoPaloQue(cartas.get(idxCartaMesa))			
					) {
					++numCartasEscalera;	
				}
				prob += probCompletarEscaleraReal(numCartasEscalera, cartasPorMostrar);
				numCartasEscalera = 1;
			}
		}
			
		return prob;
	}
	
	
	private double probCompletarEscaleraReal( int numCartasEscalera, int cartasPorMostrar){	
		
		double prob = 1.0;

		int cartasNecesarias = 5 - numCartasEscalera;
		int cartasEseTipoRestantes = cartasNecesarias;
		
		if( cartasNecesarias > cartasPorMostrar ) {
			prob = 0.0;
		}else if( cartasNecesarias > 0 &&  cartasPorMostrar > 0 ){
	
			int cartasQueNoHanSalido = 50 - ( cartasPorMostrar - 2);
			
			int combinaciones = C(cartasPorMostrar,cartasEseTipoRestantes);	// Revisar
			
			while( cartasNecesarias > 0 ) {
				prob *= (double ) cartasEseTipoRestantes / cartasQueNoHanSalido;
				--cartasEseTipoRestantes;
				--cartasQueNoHanSalido;
				--cartasNecesarias;
			}
			prob *= combinaciones;
		}
		return prob;
	}
	
}
