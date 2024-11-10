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
	private Panel panel;
	private int numContrincantes;
	private CalculoDeProbabilidades calc;
	
	public Worker( int numContrincantes, List<Carta> cartas, Panel panel) {
		this.cartas = cartas;
		this.panel = panel;
		this.numContrincantes = numContrincantes;
		this.calc = new CalculoDeProbabilidades();
	}
	
	/*
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
			panel.setProbabilidad( get() );
			panel.setErrores("Terminado");

		} catch (InterruptedException | ExecutionException e) {
			e.printStackTrace();
		} catch( Exception e) {
			e.printStackTrace();
		}
	}
	
	private int calcularProbabilidad() {
		
		calc.reiniciarDatos(cartas);
		
		sacarDatosObtenidos();
		int prob =(int)(100*calc.getProbColor()) ;
		System.out.println(".............."+ prob);
		return prob;
	}
	
	private void sacarDatosObtenidos() {
		System.out.println("-------------------------------------");
		System.out.println("Probabilidad escalera: " + calc.getProbEscalera());
		System.out.println("Probabilidad color: " + calc.getProbColor());
		System.out.println("Probabilidad FullHouse: " + calc.getProbFullHouse());
		System.out.println("Probabilidad Poker: " + calc.getProbPoker());
		System.out.println("Probabilidad escalera de color : " + calc.getProbEscaleraColor());
		System.out.println("Probabilidad escalera real: " + calc.getProbEscaleraReal());
		System.out.println("-------------------------------------");
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
