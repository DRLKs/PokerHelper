import java.util.List;
import java.util.concurrent.ExecutionException;

import javax.swing.SwingWorker;

public class Worker extends SwingWorker<Integer , Integer>{
	
	private final int TOTAL_CARTAS = 52;
	private final int CARTA_MAS_ALTA = 14;
	private final int COMBINACIONES_50_CARTAS = 1225; // No contamos nuestras cartas
	private final int CARTA_MANO_1 = 0;
	private final int CARTA_MANO_2 = 1;
	
	private List<Carta> cartas;
	private int numContrincantes;
	private CalculoDeProbabilidades calc;
	private Controlador controlador;
	
	public Worker( int numContrincantes, List<Carta> cartas, Controlador controlador) {
		this.cartas = cartas;
		this.controlador = controlador;
		this.numContrincantes = numContrincantes;
		this.calc = new CalculoDeProbabilidades();
	}
	
	/**
	 * Función que llama al SwingWorker
	 * Hilo que calcula la probabilidad en BackGround
	 */
	@Override
	protected Integer doInBackground() throws Exception {
		/*
		if( cartas.size() == 2 ) {
			return calcularProbabilidadSalida();
		}else if( cartas.size() == 5 ) {
			return 0;
		}else if(  cartas.size() == 4 ) {
			return 0;
		}else {
			return calcularProbabilidad();
		}
		*/
		return calcularProbabilidad();
		
	}
	/*
	 * Cuando termina doInBackground()
	 * get() = valor que retorna doInBackground()
	 */
	@Override
	public void done() {
		
		try {
			String nuevaDecision;
			int decision = get();
			
			switch (decision) {
			case 0: 
				nuevaDecision = "Apostar";
				break;
			case 1:
				nuevaDecision = "Salir";
				break;
			default:
				nuevaDecision = "Error";
			}
			
			controlador.setDecision(nuevaDecision);

		} catch (InterruptedException | ExecutionException e) {
			e.printStackTrace();
		} catch( Exception e) {
			e.printStackTrace();
		}
	}
	
	/**
	 * Función llamada en segundo plano, esta hace todos los calculos probabilísticos
	 * 
	 * <b> POR TERMINAR </b>
	 * 
	 * @return Devuelve un entero que identifica a una decisión que debe tomar el jugador
	 */
	private int calcularProbabilidad() {
		
		calc.reiniciarDatos(cartas , numContrincantes);
		
		sacarDatosObtenidos();
		int prob =(int)(100*calc.getProbEscaleraColor()) ;
		System.out.println(".............."+ prob);
		return 1;	
	}
	
	private void sacarDatosObtenidos() {
		System.out.println("-------------------------------------");
		System.out.println("Probabilidad pareja: " + calc.getProbPareja());
		
		System.out.println("Probabilidad trio: " + calc.getProbTrio());
		System.out.println("Probabilidad escalera: " + calc.getProbEscalera());
		System.out.println("Probabilidad color: " + calc.getProbColor());
		System.out.println("Probabilidad FullHouse: " + calc.getProbFullHouse());
		System.out.println("Probabilidad Poker: " + calc.getProbPoker());
		System.out.println("Probabilidad escalera de color : " + calc.getProbEscaleraColor());
		System.out.println("Probabilidad escalera real: " + calc.getProbEscaleraReal());

		System.out.println("Probabilidad pareja contr: " + calc.getProbParejaCont());
		System.out.println("Probabilidad trio contr: " + calc.getProbTrioCont());
		System.out.println("Probabilidad escalera contr: " + calc.getProbEscaleraCont());
		System.out.println("Probabilidad color contr contr: " + calc.getProbColorCont());
		System.out.println("Probabilidad FullHouse contr: " + calc.getProbFullHouseCont());
		System.out.println("Probabilidad Poker contr: " + calc.getProbPokerCont());
		System.out.println("Probabilidad escalera de color contr: " + calc.getProbEscaleraColorCont());
		System.out.println("Probabilidad escalera real contr: " + calc.getProbEscaleraRealCont());
		
		System.out.println("-------------------------------------");
		
		
		controlador.setProbabilidad(0, (int) (calc.getProbPareja() 		  * 100) );
		controlador.setProbabilidad(1, (int) (calc.getProbTrio() 		  * 100) );
		controlador.setProbabilidad(2, (int) (calc.getProbEscalera() 	  * 100) );
		controlador.setProbabilidad(3, (int) (calc.getProbColor() 		  * 100) );
		controlador.setProbabilidad(4, (int) (calc.getProbFullHouse() 	  * 100) );
		controlador.setProbabilidad(5, (int) (calc.getProbPoker() 		  * 100) );
		controlador.setProbabilidad(6, (int) (calc.getProbEscaleraColor() * 100) );
		controlador.setProbabilidad(7, (int) (calc.getProbEscaleraReal()  * 100) );

		
	}
	
	private double calcularProbabilidadMejoresCartasIndividualesContrincantes() {	// Asumimos que nadie va a sacar Escalera de color ni real, que los calculos ya son suficientemente complejos
		
		double probabilidad = 0;
		
		if( cartas.get( CARTA_MANO_1 ).mismoNumeroQue(cartas.get( CARTA_MANO_2 )) ) {	 
			
			int cantidad_de_posibles_numeros_mas_altos = CARTA_MAS_ALTA - cartas.get( CARTA_MANO_1 ).getNumero();
			int numeroManosMejores = calc.C(4, 2) * (cantidad_de_posibles_numeros_mas_altos + 1 );
			int combinacionesCartas = calc.C( TOTAL_CARTAS - cartas.size() , 2 );
			
			double prob_mano_mejor_contrincante = (double) numeroManosMejores / combinacionesCartas;
			double prob_mejores_manos_algún_contrincante = Math.pow(1 - prob_mano_mejor_contrincante, numContrincantes );
			
			probabilidad = (int) (prob_mejores_manos_algún_contrincante * 100);
			//System.out.println(probabilidad + "-----" + prob_mano_mejor_contrincante + "-----" + prob_mejores_manos_algún_contrincante * 100 + "--" + numeroManosMejores + "----" + combinacionesCartas);
			
		}else if( cartas.get( CARTA_MANO_1 ).mismoPaloQue( cartas.get( CARTA_MANO_2 ) ) ) {	
			
			
		}else if( cartas.get( CARTA_MANO_1 ).puedenHacerEscalera( cartas.get( CARTA_MANO_2 ) ) ) {
				
		}else {
			
			int numeroCartaAlta;
			if( cartas.get( CARTA_MANO_1 ).masAltaQue( cartas.get(CARTA_MANO_2) ) ) {
				numeroCartaAlta = cartas.get(CARTA_MANO_1).getNumero();
			}else {
				numeroCartaAlta = cartas.get(CARTA_MANO_2).getNumero();
			}
			
			
			
		}
		
		return probabilidad;
	}
}
