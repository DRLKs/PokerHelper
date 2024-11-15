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
	
	private final int MAX_CARTAS_VISIBLES = 7;
	private final int IDX_CARTA_MANO_1 = 0;
	private final int IDX_CARTA_MANO_2 = 1;
	private final int NUM_CARTAS_NUNCA_VES = 45;
	private final int NUM_CARTAS_NECESARIAS__ESCALERA = 5;

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
	
	public int C( int n ,int k) { // COMBINACIONES
		
		if (k > n || k < 0) return 0; // No existe combinatoria si k > n
        if (k == 0 || k == n ) return 1;
        // La combinación es simétrica, C(n, k) = C(n, n-k)
        if (k > n - k) {
            k = n - k;
        }
        int ini = 1 + n - k; 
        int resultado = 1;
        for (int i = ini; i <= n; ++i) {
            resultado *= i;
        }
        for(int i = 2 ; i <= k ; ++i) {
        	resultado /= i;
        }

        return resultado;
	}
	
	/* 
	 * Estas Funciones calcularán las posibilidades de obtener color
	 * numCartasColor: 	son las cartas de **** que tienes en tu mano
	 * 
	 * 	CABE DESTACAR: El número de jugadores no interviene en las probabilidades que haya de obtener tu mano deseada
	 * 
	 * */
	private double completarColor( List<Carta> cartas ){
		
		double prob;
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
			
			prob = distribucionHiperGeometrica(5-numCartasColor, 13-numCartasColor, cartasPorMostrar);
		}else {
			
			prob = 0.0;
			for( int idxCMano = 0 ; idxCMano < 2 ; ++idxCMano ) {	// Comrpobar que nuestras cartas participen en estos datos
				for( int idx = 0; idx < 2 ; ++idx ) {
					for( int i = 2; i < numCartas ; ++i ) {
						if( cartas.get(i).mismoPaloQue(cartas.get(idx)) ) {
							++numCartasColor;
						}
					}
					prob += distribucionHiperGeometrica(5-numCartasColor, 13-numCartasColor, cartasPorMostrar);
					numCartasColor = 1;
				}
			}
		}

		return prob;
	}
	
	/*
	 * Al encontrarnos con varias cartas, hay diferentes posibles escaleras:
	 * 	
	 * 	
	 * 	
	 */
	private double completarEscalera( List<Carta> cartas ){
			
		double prob = 0.0;
		int numCartas = cartas.size();
		int cartasPorMostrar = MAX_CARTAS_VISIBLES - numCartas;
		int numCartaMano1 = cartas.get(IDX_CARTA_MANO_1).getNumero();
		int numCartaMano2 = cartas.get(IDX_CARTA_MANO_2).getNumero();
		int arrayInicio;
		int arrayFinal;
		
		/*
		 * El array "numerosEscalera" tendrá un tamaño igual a 14
		 *  En este array podremos comprobar si tenemos un número, por ejemplo:
		 *  
		 *  Comprobamos que tenemos el AS -> numerosEscalera[14] == 1
		 *  		TRUE -> Tenemos el AS
		 *  		FALSE -> No tenemos el AS
		 *  El AS al tener asociado el número 14, lo podremos comprobar en el lugar 14 y a la 1
		 *  No usaremos la posición 0
		 */
		int[] numerosEscalera;	
		int[] arrayFronterasEscalera;
		
		// Nuestras 2 cartas pueden hacer escalera
		if( cartas.get(0).puedenHacerEscalera( cartas.get(1) )) {	
			numerosEscalera = new int[15];
			
			numerosEscalera[ numCartaMano1 ] = 1;
			numerosEscalera[ numCartaMano2 ] = 1;
			
			arrayFronterasEscalera = encontrarFronterasEscaleras(numCartaMano1, numCartaMano2);
			arrayInicio = arrayFronterasEscalera[0];
			arrayFinal  = arrayFronterasEscalera[1];
			
			for( int idx = 2; idx < numCartas ; ++idx ) {
				numerosEscalera[ cartas.get(idx).getNumero() ] = 1;
			}
			if( numerosEscalera[14] == 1 ) {	// Está el AS, 14-1=13
				numerosEscalera[1] = 1;
			}
			prob = probCompletarEscalera(numerosEscalera, cartasPorMostrar, arrayInicio, arrayFinal);
			
		}else {	// Nuestras 2 cartas no pueden hacer escalera de
			for( int idxCMano = 0 ; idxCMano < 2 ; ++idxCMano ) {	
				numerosEscalera = new int[15];
				numerosEscalera[ cartas.get(idxCMano).getNumero() ] = 1;	// Añadimos la carta de la mano
				arrayFronterasEscalera = encontrarFronterasEscaleras(cartas.get(idxCMano).getNumero(), -1);
				arrayInicio = arrayFronterasEscalera[0];
				arrayFinal  = arrayFronterasEscalera[1];
				for( int idx = 2; idx < numCartas ; ++idx ) {
					if( 
						cartas.get(idxCMano).mismoPaloQue(cartas.get(idx))
						) {
						numerosEscalera[ cartas.get(idx).getNumero() ] = 1;
					}
					if( numerosEscalera[14] == 1 ) {	// Está el AS, 14-1=13
						numerosEscalera[1] = 1;
					}
					prob += probCompletarEscalera(numerosEscalera, cartasPorMostrar, arrayInicio, arrayFinal);
				}
			}
		}
		
		return prob;
			
		
	}
	
	private double probCompletarEscalera( int[] numerosEscalera, int cartasPorMostrar, int arrayInicio, int arrayFinal ){
		double prob = 0.0;
		int numCartasEscalera;
		for( int idx1 = arrayInicio ; idx1 <= arrayFinal - 4 ; ++idx1 ) {
			numCartasEscalera = 0;
			for( int idx = idx1 ; idx <= idx1+4 ; ++idx) {
				if( numerosEscalera[idx] == 1 ) {
					++numCartasEscalera;
					if( numCartasEscalera >= 5 ) {
						return 1.0;
					}
				}
			}
			prob += probCompletarEstaEscalera(numCartasEscalera, cartasPorMostrar);
		}
		return prob;
	}
	
	private double probCompletarEstaEscalera( int numCartasEscalera, int cartasPorMostrar ){
		
		double prob = 0.0;
		
		int cartasNecesarias = NUM_CARTAS_NECESARIAS__ESCALERA - numCartasEscalera;
		int cartasEseTipoRestantes = cartasNecesarias * 4;
		
		if( cartasNecesarias > cartasPorMostrar ) {
			prob = 0.0;
		}else if( cartasNecesarias > 0 &&  cartasPorMostrar > 0 ){
			if( cartasNecesarias > 2 ) {
				prob = C(cartasEseTipoRestantes,cartasPorMostrar);	// Revisar
			}else {
				prob = 1.0;
			}
			int cartasQueNoHanSalido = 45 + cartasPorMostrar;
			
			
			while( cartasNecesarias > 0 ) {
				prob *= (double ) cartasEseTipoRestantes / cartasQueNoHanSalido;
				--cartasEseTipoRestantes;
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
	
	private double completarPoker( List<Carta> cartas ) {	// ARREGLAR
		
		double prob = 0.0;
		
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
			prob = distribucionBinomial( 4 - cttCartasPoker , cartasPorMostrar);
		}else {
			prob = distribucionBinomial( 1 , cartasPorMostrar) * 2;	// FATAL
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
			
			long combinacionesPosibles = C(cartasPorMostrar,cartasNecesarias);
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
		int cartasPorMostrar = MAX_CARTAS_VISIBLES - numCartas;
		int numCartaMano1 = cartas.get(IDX_CARTA_MANO_1).getNumero();
		int numCartaMano2 = cartas.get(IDX_CARTA_MANO_2).getNumero();
		int arrayInicio;
		int arrayFinal;
		
		/*
		 * El array "numerosEscaleraColor" tendrá un tamaño igual a 14
		 *  En este array podremos comprobar si tenemos un número, por ejemplo:
		 *  
		 *  Comprobamos que tenemos el AS -> numerosEscaleraColor[14] == 1
		 *  		TRUE -> Tenemos el AS
		 *  		FALSE -> No tenemos el AS
		 *  El AS al tener asociado el número 14, lo podremos comprobar en el lugar 14 y a la 1
		 *  No usaremos la posición 0
		 */
		int[] numerosEscaleraColor;	
		int[] arrayFronterasEscalera;
		
		// Nuestras 2 pueden hacer escalera color
		if( cartas.get(0).puedenHacerEscalera( cartas.get(1) ) &&  cartas.get(0).mismoPaloQue( cartas.get(1))) {	
			numerosEscaleraColor = new int[15];
			
			numerosEscaleraColor[ numCartaMano1 ] = 1;
			numerosEscaleraColor[ numCartaMano2 ] = 1;
			
			arrayFronterasEscalera = encontrarFronterasEscaleras(numCartaMano1, numCartaMano2);
			arrayInicio = arrayFronterasEscalera[0];
			arrayFinal  = arrayFronterasEscalera[1];
			
			for( int idx = 2; idx < numCartas ; ++idx ) {
				if( cartas.get(0).mismoPaloQue(cartas.get(idx)) ) {		// Que no puedan hacer escalera no importa, se filtra luego
					numerosEscaleraColor[ cartas.get(idx).getNumero() ] = 1;
				}
			}
			if( numerosEscaleraColor[14] == 1 ) {	// Está el AS, 14-1=13
				numerosEscaleraColor[1] = 1;
			}
			prob = probCompletarEscaleraColor(numerosEscaleraColor, cartasPorMostrar, arrayInicio, arrayFinal);
			
		}else {	// Nuestras 2 cartas no pueden hacer escalera de color
			for( int idxCMano = 0 ; idxCMano < 2 ; ++idxCMano ) {	
				numerosEscaleraColor = new int[15];
				numerosEscaleraColor[ cartas.get(idxCMano).getNumero() ] = 1;	// Añadimos la carta de la mano
				arrayFronterasEscalera = encontrarFronterasEscaleras(cartas.get(idxCMano).getNumero(), -1);
				arrayInicio = arrayFronterasEscalera[0];
				arrayFinal  = arrayFronterasEscalera[1];
				for( int idx = 2; idx < numCartas ; ++idx ) {
					if( 
						cartas.get(idxCMano).mismoPaloQue(cartas.get(idx))
						) {
						numerosEscaleraColor[ cartas.get(idx).getNumero() ] = 1;
					}
					if( numerosEscaleraColor[14] == 1 ) {	// Está el AS, 14-1=13
						numerosEscaleraColor[1] = 1;
					}
					prob += probCompletarEscaleraColor(numerosEscaleraColor, cartasPorMostrar, arrayInicio, arrayFinal);
				}
			}
		}
		
		return prob;
	}
	/*
	 * Función utilizada por las funciones: completarEscalera, completarEscaleraColor
	 * Dados 2 números o 1, devuelve un array de dimensión 2, con los límites a estudiar en el array "numerosEscaleraColor"
	 * 	arrayFronterasEscalera[0] -> Límite inferior
	 * 	arrayFronterasEscalera[1] -> Límite superior
	 */
	private int[] encontrarFronterasEscaleras( int numCarta1, int numCarta2 ) {
		int[] arrayFronterasEscalera = new int[2];
		int arrayFinal;
		int arrayInicio;
		if( numCarta1 == 14 || numCarta2 == 14 ) {	// se puede optimizar
			arrayInicio = 1;
			arrayFinal = 14;
		}else {
		
			arrayInicio = numCarta1 - 4;
			if( numCarta2 != -1 && arrayInicio < numCarta2 - 4 ) {
				arrayInicio = numCarta2 - 4;
				arrayFinal = numCarta2 + 4;
			}else {
				arrayFinal = numCarta1 + 4;
			}
			if( arrayFinal > 14 ) {
				arrayFinal = 14;
			}
			if( arrayInicio < 1 ) {
				arrayInicio = 1;
			}
		}
		arrayFronterasEscalera[0] = arrayInicio;
		arrayFronterasEscalera[1] = arrayFinal;
		
		return arrayFronterasEscalera;
	}
	
	private double probCompletarEscaleraColor( int[] numerosEscaleraColor, int cartasPorMostrar, int arrayInicio, int arrayFinal ){
		double prob = 0.0;
		int numCartasNecesarias;
		int numCartasEscalera;
		for( int idx1 = arrayInicio ; idx1 <= arrayFinal - 4 ; ++idx1 ) {
			numCartasEscalera = 0;
			for( int idx = idx1 ; idx <= idx1+4 ; ++idx) {
				if( numerosEscaleraColor[idx] == 1 ) {
					++numCartasEscalera;
					if( numCartasEscalera >= 5 ) {
						return 1.0;
					}
				}
			}
			numCartasNecesarias = 5 - numCartasEscalera;
			prob += distribucionHiperGeometrica( numCartasNecesarias, numCartasNecesarias*4,  cartasPorMostrar );
		}
		return prob;
	}

	/*
	 * FUNCIONA -> QUIZÁS ARREGLAR COMBINACIONES
	 */
	private double completarEscaleraReal( List<Carta> cartas ){
		
		double prob = 0.0;
		int numCartas = cartas.size();
		int cartasPorMostrar = MAX_CARTAS_VISIBLES - numCartas;
		int numCartasEscalera = 1;
		boolean cartaMano1Posible = false;
		boolean cartaMano2Posible = false;
		/*
		 * Para hacerlo más eficiente
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
			numCartasEscalera = 2;
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
			}
			int cartasNecesarias = 5 - numCartasEscalera;
			prob += distribucionHiperGeometrica(cartasNecesarias,cartasNecesarias, cartasPorMostrar);	// Las cartas necesarias son las mismas, a las válidas
			numCartasEscalera = 1;
		}
			
		return prob;
	}
	
	/*
	 * Mediante la distribución HiperGeométrica
	 * Devolverá la probabilidad de que la mano X  se complete
	 */
	private double distribucionHiperGeometrica( int cartasNecesarias, int cartasValidasRestantes, int cartasPorMostrar) {
		double prob;
		if( cartasNecesarias <= 0 )
			prob = 1.0;
		else if( cartasNecesarias > cartasPorMostrar ) {
			prob = 0.0;
		}else{
			int combinacionesPosibles = C( NUM_CARTAS_NUNCA_VES + cartasPorMostrar , cartasPorMostrar );
			int combinacionesFormasSalirCartasNecesarias = C( cartasValidasRestantes , cartasNecesarias );
			int combinacionesDemasCartas = C( NUM_CARTAS_NUNCA_VES + (cartasPorMostrar-cartasNecesarias), (cartasPorMostrar-cartasNecesarias));
			
			prob = (double) (combinacionesFormasSalirCartasNecesarias * combinacionesDemasCartas) / combinacionesPosibles;
		}
		
		return prob;
	}
	
	
	private double distribucionBinomial(int cartasNecesarias, int cartasPorMostrar) {
		
		int cartasQueNoHanSalido = 45 + cartasPorMostrar;			//N = cartasQueNoHanSalido ; K = Cartas Necesarias
		
		long combinacionesNK = C(cartasQueNoHanSalido, cartasNecesarias);
		double p = cartasNecesarias / cartasQueNoHanSalido;
		
		return (double) combinacionesNK * Math.pow(p, cartasNecesarias) * Math.pow(1.0-p, cartasQueNoHanSalido-cartasNecesarias);
	}
	/*
	 * ##############################################################################################################
	 * 								CALCULAR PROBABILIDAD DE LOS CONTRINCANTES
	 * ##############################################################################################################
	 */
}
